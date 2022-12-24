use regex::Regex;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

// 运行的方法:
// cargo run
// 然后再浏览器输入your IP addreess:7878即可
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    // --snip--
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {} got a job; executing.", id);

            job();
        });

        Worker { id, thread }
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    // --snip--

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}
fn main() {
    start();
}

fn start() {
    // TODO2:完成你的IP地址的查找
    let listener = TcpListener::bind("192.168.60.100:7878").unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // 问题是不知道他要获取什么报文？
        // 报文的格式是什么？
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    // TODO1:完成你的正则表达式
    // 从 "GET /1.png HTTP/1.1" 中找到 1.png
    // r"GET (.*?) HTTP/1.1"
    let re = Regex::new(r"GET(\s)(.*)(\s)HTTP/1.1").unwrap();
    let str = String::from_utf8_lossy(&buffer);
    // 输出看看是什么
    print!("报文如下\n");
    print!("{}", str);

    // 数据包是这样的
    // Worker 2 got a job; executing.
    // GET / HTTP/1.1
    // Host: 192.168.60.100:7878
    // Connection: keep-alive
    // Cache-Control: max-age=0
    // Upgrade-Insecure-Requests: 1
    // User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36
    // Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9
    // Accept-Encoding: gzip, deflate
    // Accept-Language: zh-CN,zh;q=0.9

    if buffer.starts_with(get) {
        let contents = fs::read_to_string("hello.html").unwrap();

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else {

        for cap in re.captures_iter(&*str) {
            // cap[0] 是原文
            print!("cap[0]:{}\n", &cap[0]);
            // cap[1] 是匹配到的结果
            print!("cap[1]:{}\n", &cap[1]);
            // [1..] 是字符串截取，从1到最后
            print!("cap[1][1..]:{}\n", &cap[1][1..]);
            print!("cap[2][1..]:{}\n", &cap[2][1..]);
            let path = &cap[2][1..];
            println!("path:{}", path);
            match fs::read(path) {
                Ok(s) => {
                    if path.ends_with(".png") {
                        let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type:image/x-icon\r\n\r\n", s.len());
                        stream.write(response.as_bytes()).unwrap();
                        stream.write_all(&s[..]).unwrap();
                        stream.flush().unwrap();
                    } else {
                        let response =
                            format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", s.len());
                        stream.write(response.as_bytes()).unwrap();
                        stream.write_all(&s[..]).unwrap();
                        stream.flush().unwrap();
                    }
                }
                Err(s) => {
                    let status_line = "HTTP/1.1 404 NOT FOUND";
                    let contents = fs::read_to_string("404.html").unwrap();

                    let response = format!(
                        "{}\r\nContent-Length: {}\r\n\r\n{}",
                        status_line,
                        contents.len(),
                        contents
                    );

                    stream.write(response.as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
            }
        }
    }
}
