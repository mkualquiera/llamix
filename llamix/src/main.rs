use json::{object, JsonValue};

use ansi_term::Colour::Cyan;

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
enum ChatMessage {
    System(String),
    User(String),
    Assistant(String),  
}

impl ChatMessage {
    fn label(&self) -> &str {
        match self {
            ChatMessage::System(_) => "system",
            ChatMessage::User(_) => "user",
            ChatMessage::Assistant(_) => "assistant",
        }
    }
    fn content(&self) -> &str {
        match self {
            ChatMessage::System(content) => content,
            ChatMessage::User(content) => content,
            ChatMessage::Assistant(content) => content,
        }
    }
}

struct MessageList(Vec<ChatMessage>);

impl MessageList {
    fn to_json(&self) -> JsonValue {
        let mut json = json::JsonValue::new_array();
        for message in &self.0 {
            let _ = json.push(object! {
                role: message.label(),
                content: message.content(),
            });
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

    fn complete(&self, messages: &MessageList) -> ChatMessage {
        let json = object! {
            model: self.model.clone(),
            messages: messages.to_json(),
            stream: false
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
            "assistant" => ChatMessage::Assistant(content.to_string()),
            _ => panic!("unknown role"),
        }
    }
}

fn main() {

    // Mount /proc using busybox
    EzCommand::new("/bin/busybox")
        .arg("mount")
        .arg("-t")
        .arg("proc")
        .arg("proc")
        .arg("/proc")
        .spawn()
        .wait()
        .expect("failed to wait on child");

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

    // Clear the screen by adding a bunch of newlines
    for _ in 0..100 {
        println!();
    }

    println!("Hello from llamix!!");

    //EzCommand::new("/bin/busybox")
    //    .arg("sh")
    //    .spawn()
    //    .wait()
    //    .expect("failed to wait on child");

    let ollama = Ollama::new("http://10.0.2.2:11434/api/chat", "llama3.1");

    let mut messages = MessageList(vec![]);
    
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        // echo the input
        messages.0.push(ChatMessage::User(input.trim().to_string()));
        let response = ollama.complete(&messages);
        messages.0.push(response.clone());
        println!("{}", Cyan.paint(response.content()));
    }
}
