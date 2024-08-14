use core::panic;

use json::{array, object, JsonValue};

use ansi_term::Colour::{Cyan, Green};

struct EzCommand {
    cmd: std::process::Command,
}

impl EzCommand {
    fn new(cmd: &str) -> Self {
        let command = std::process::Command::new(cmd);
        Self { cmd: command }
    }

    fn arg(&mut self, arg: &str) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    fn cap_output(&mut self) -> &mut Self {
        self.cmd.stdout(std::process::Stdio::piped());
        self.cmd.stderr(std::process::Stdio::piped());
        self
    }

    fn spawn(&mut self) -> std::process::Child {
        self.cmd.spawn().expect("failed to execute process")
    }
}

/// make a request using busybox wget
fn request(url: &str, data: &str) -> String {
    //let child = EzCommand::new("/bin/busybox")
    //    .arg("curl")
    //    .arg("-d")
    //    .arg(data)
    //    .arg(url)
    //    .spawn();
    let child = EzCommand::new("/bin/busybox")
        .arg("wget")
        .arg("-q")
        .arg("--post-data")
        .arg(data)
        .arg("-O")
        .arg("-")
        .arg(url)
        .cap_output()
        .spawn();

    let output = child.wait_with_output().expect("failed to wait on child");
    let out = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    out
}

struct Ollama {
    url: String,
    model: String,
}

#[derive(Clone)]
struct ToolCall {
    name: String,
    arguments: JsonValue,
}

#[derive(Clone)]
struct ToolCallResult {
    name: String,
    result: String
}

#[derive(Clone)]
enum ChatMessage {
    System(String),
    User(String),
    Assistant(String, Option<Vec<ToolCall>>),  
    Tool(Vec<ToolCallResult>)
}

impl ChatMessage {
    fn label(&self) -> &str {
        match self {
            ChatMessage::System(_) => "system",
            ChatMessage::User(_) => "user",
            ChatMessage::Assistant(_, _) => "assistant",
            ChatMessage::Tool(_) => "tool",
        }
    }
    fn content(&self) -> String {
        match self {
            ChatMessage::System(content) => content.clone(),
            ChatMessage::User(content) => content.clone(),
            ChatMessage::Assistant(content, _) => content.clone(),
            ChatMessage::Tool(tool_results) => {
                let mut result = String::new();
                for tool_result in tool_results {
                    result.push_str(&format!("{}: {}\n", tool_result.name, tool_result.result));
                }
                result
            },
        }
    }
}

#[derive(Clone)]
struct MessageList(Vec<ChatMessage>);

impl MessageList {
    fn to_json(&self) -> JsonValue {
        let mut json = json::JsonValue::new_array();
        for message in &self.0 {
            let mut message_json = object! {
                role: message.label(),
                content: message.content(),
            };
            if let ChatMessage::Assistant(_, Some(tool_calls)) = message {
                let mut tool_calls_json = json::JsonValue::new_array();
                for tool_call in tool_calls {
                    let tool_call_json = object! {
                        function: {
                            name: tool_call.name.clone(),
                            arguments: tool_call.arguments.clone(),
                        }
                    };
                    _ = tool_calls_json.push(tool_call_json);
                }
                message_json["tool_calls"] = tool_calls_json;
            }
            let _ = json.push(message_json);
        }
        json
    }
}

impl Ollama {
    fn new(url: &str, model: &str) -> Self {
        Self {
            url: url.to_string(),
            model: model.to_string(),
        }
    }

    fn complete(&self, messages: &MessageList, tools: JsonValue) -> ChatMessage {
        let json = object! {
            model: self.model.clone(),
            messages: messages.to_json(),
            stream: false,
            tools: tools,
        };

        let response = request(&self.url, &json.dump());
        //println!(">{}<", response);
        let parsed = json::parse(&response).expect("failed to parse response");
        let message = &parsed["message"];
        let role = message["role"].as_str().expect("role not found");
        let content = message["content"].as_str().expect("content not found");
        
        match role {
            "system" => ChatMessage::System(content.to_string()),
            "user" => ChatMessage::User(content.to_string()),
            "assistant" => {
                let tool_calls = match message["tool_calls"] {
                    JsonValue::Array(ref tool_calls) => {
                        let mut calls = vec![];
                        for tool_call in tool_calls {
                            println!("{}", tool_call["function"]);
                            let name = tool_call["function"]["name"].as_str().expect("tool name not found");
                            let arguments = tool_call["function"]["arguments"].clone();
                            calls.push(ToolCall {
                                name: name.to_string(),
                                arguments: arguments,
                            });
                        }
                        Some(calls)
                    },
                    _ => None,
                };
                ChatMessage::Assistant(content.to_string(), tool_calls)
            }
            "tool" => panic!("Model returned tool message"),
            _ => panic!("unknown role"),
        }
    }
}

fn main() {

    let args = std::env::args().collect::<Vec<String>>();

    // See if we are in rw-mode by checking if we have a second argument
    if args.len() < 2 {
        println!("Hello from llamix in ro!!");

        // Mount a tmpfs on /mnt to make it writable
        EzCommand::new("/bin/busybox")
            .arg("mount")
            .arg("-t")
            .arg("tmpfs")
            .arg("tmpfs")
            .arg("/mnt")
            .spawn()
            .wait()
            .expect("failed to wait on child");

        // Copy /bin to /mnt/bin
        EzCommand::new("/bin/busybox")
            .arg("cp")
            .arg("-r")
            .arg("/bin")
            .arg("/mnt")
            .spawn()
            .wait()
            .expect("failed to wait on child");

        // Set up /proc 
        // mkdir
        EzCommand::new("/bin/busybox")
            .arg("mkdir")
            .arg("/mnt/proc")
            .spawn()
            .wait()
            .expect("failed to wait on child");
        EzCommand::new("/bin/busybox")
            .arg("mount")
            .arg("-t")
            .arg("proc")
            .arg("proc")
            .arg("/mnt/proc")
            .spawn()
            .wait()
            .expect("failed to wait on child");

        // Set up /sys
        // mkdir
        EzCommand::new("/bin/busybox")
            .arg("mkdir")
            .arg("/mnt/sys")
            .spawn()
            .wait()
            .expect("failed to wait on child");
        EzCommand::new("/bin/busybox")
            .arg("mount")
            .arg("-t")
            .arg("sysfs")
            .arg("sysfs")
            .arg("/mnt/sys")
            .spawn()
            .wait()
            .expect("failed to wait on child");

        // Set up /dev
        // mkdir
        EzCommand::new("/bin/busybox")
            .arg("mkdir")
            .arg("/mnt/dev")
            .spawn()
            .wait()
            .expect("failed to wait on child");
        EzCommand::new("/bin/busybox")
            .arg("mount")
            .arg("-t")
            .arg("devtmpfs")
            .arg("devtmpfs")
            .arg("/mnt/dev")
            .spawn()
            .wait()
            .expect("failed to wait on child");

        // Create /usr/bin, /sbin using busybox sh
        EzCommand::new("/bin/busybox")
            .arg("sh")
            .arg("-c")
            .arg("/bin/busybox mkdir -p /mnt/usr/bin /mnt/usr/sbin /mnt/sbin")
            .spawn()
            .wait()
            .expect("failed to wait on child");
    
        // Chroot into /mnt and run /bin/llamix --rw
        EzCommand::new("/bin/busybox")
            .arg("chroot")
            .arg("/mnt")
            .arg("/bin/llamix")
            .arg("--rw")
            .spawn()
            .wait()
            .expect("failed to wait on child");

        

        return;
    }

    println!("Hello from llamix in rw!!");

    // Set up networking
    EzCommand::new("/bin/busybox")
        .arg("ip")
        .arg("link")
        .arg("set")
        .arg("eth0")
        .arg("up")
        .spawn().wait().expect("failed to wait on child");
    EzCommand::new("/bin/busybox")
        .arg("ip")
        .arg("addr")
        .arg("add")
        .arg("10.0.2.15/24")
        .arg("dev")
        .arg("eth0")
        .spawn().wait().expect("failed to wait on child");

    // Set gateway 10.0.2.2
    EzCommand::new("/bin/busybox")
        .arg("ip")
        .arg("route")
        .arg("add")
        .arg("default")
        .arg("via")
        .arg("10.0.2.2")
        .spawn().wait().expect("failed to wait on child");


    // Install busybox with --install
    EzCommand::new("/bin/busybox")
        .arg("--install")
        .spawn()
        .wait()
        .expect("failed to wait on child");

    // Clear the screen by adding a bunch of newlines

    println!("Hello from llamix!!");

    //EzCommand::new("/bin/busybox")
    //    .arg("sh")
    //    .spawn()
    //    .wait()
    //    .expect("failed to wait on child");

    let ollama = Ollama::new("http://10.0.2.2:11434/api/chat", "llama3.1");

    let mut messages = MessageList(vec![ChatMessage::System("You are an LLM powered OS called llamix, running in a barebones unix environment as pid 1. You can chat with the user and complete tasks.".to_string())]);

    let mut user_has_control = true;
    
    loop {
        let mut input = String::new();
        if user_has_control {
            std::io::stdin().read_line(&mut input).unwrap();
            // echo the input
            messages.0.push(ChatMessage::User(input.trim().to_string()));
            user_has_control = false;
        }
        let tools = array![{
            type: "function",
            function: {
                name: "sh",
                description: "Run a command using sh and get the output",
                parameters: {
                    type: "object",
                    properties: {
                        command: {
                            type: "string",
                            description: "The command to run"
                        }
                    },
                    required: ["command"]
                }
            }
        }, 
        {
            type: "function",
            function: {
                name: "say",
                description: "Say something to the user",
                parameters: {
                    type: "object",
                    properties: {
                        message: {
                            type: "string",
                            description: "The message to say"
                        }
                    },
                    required: ["message"]
                }
            }
        }
        ];

        let mut these_messages = messages.clone();
        //these_messages.0.push(ChatMessage::System("Use the `return` to return control to the user.".to_string()));

        let response = ollama.complete(&these_messages, tools);
        messages.0.push(response.clone());
        println!("{}", Cyan.paint(response.content()));
        if let ChatMessage::Assistant(_, Some(tool_calls)) = response {
            let mut tool_results = vec![];
            for tool_call in tool_calls {
                // ensure tool name is busybox_sh
                if tool_call.name == "sh" {
                    let command = tool_call.arguments["command"].as_str().expect("command not found");
                    let output = EzCommand::new("/bin/busybox")
                        .arg("sh")
                        .arg("-c")
                        .arg(command)
                        .cap_output()
                        .spawn()
                        .wait_with_output()
                        .expect("failed to wait on child");
                    let result = String::from_utf8_lossy(&output.stdout);
                    let result_stderr = String::from_utf8_lossy(&output.stderr);
                    let result = format!("{}\n{}", result, result_stderr);
                    println!("{}: {}", Green.paint(&tool_call.name), Green.paint(&result));
                    tool_results.push(ToolCallResult {
                        name: tool_call.name.clone(),
                        result: result,
                    });
                }
                else if tool_call.name == "say" {
                    let message = tool_call.arguments["message"].as_str().expect("message not found");
                    println!("{}: {}", Green.paint(&tool_call.name), Green.paint(message));
                    tool_results.push(ToolCallResult {
                        name: tool_call.name.clone(),
                        result: message.to_string(),
                    });
                    user_has_control = true;
                }
                else {
                    panic!("Unknown tool name");
                }
            }
            messages.0.push(ChatMessage::Tool(tool_results));
        }
        else {
            user_has_control = true;
        }

    }
}
