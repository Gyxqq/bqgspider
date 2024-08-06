use regex::Regex;
use reqwest::{self};
use std::{
    fs::File,
    io::{stdin, Write},
    sync::{Arc, Mutex},
};
use tokio::{self, sync::Semaphore};

async fn get_content(url: &str) -> Result<String, reqwest::Error> {
    let mut retry_time = 0;
    loop {
        if retry_time > 10 {
            println!("Retry too many times, exit");
            return Ok(format!("Retry too many times, exit url {}", url));
        }
        let body = reqwest::get(url).await;
        match body {
            Ok(response) => {
                let body = response.text().await;
                match body {
                    Ok(text) => {
                        let mut content_string = String::new();
                        let mut reg = Regex::new("<h1 class=\"wap_none\">[^<]+").unwrap();
                        let title = reg.captures(text.as_str());
                        match title {
                            Some(title) => {
                                println!("Title: {}", &title[0][21..]);
                                content_string.push_str(&title[0][21..]);
                            }
                            None => {
                                println!("No title error url {}", url);
                                retry_time += 1;
                                continue;
                            }
                        }
                        reg = Regex::new("[^>]+<br /><br />").unwrap();
                        let contents = reg.captures_iter(text.as_str());

                        for content in contents {
                            // 跳过最后一个
                            if content[0].contains("https://m.bqgui.cc") {
                                continue;
                            }
                            // println!("{}", &content[0]);
                            content_string.push_str(&content[0].replace("<br /><br />", "\n"));
                        }
                        content_string.push_str("\n\n");
                        return Ok(content_string);
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        retry_time += 1;
    }
}
#[tokio::main]
async fn main() {
    // 输入小说地址
    // https://www.bqgui.cc/book/12272/
    let mut url = String::new();
    print!("Please input the url of the novel: ");
    std::io::stdout().flush().unwrap();
    match stdin().read_line(&mut url) {
        Ok(_) => {
            url = url.trim().to_string();
        }
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    }
    print!("plaese input thread number: ");
    std::io::stdout().flush().unwrap();
    let mut thread_number = String::new();
    match stdin().read_line(&mut thread_number) {
        Ok(_) => {
            thread_number = thread_number.trim().to_string();
        }
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    }
    let thread_number: usize = match thread_number.parse() {
        Ok(thread_number) => thread_number,
        Err(e) => {
            println!("set default thread number 20");
            20
        }
    };
    println!("Url: {}", &url);
    let body = reqwest::get(&url).await;
    match body {
        Ok(response) => {
            let body = response.text().await;
            match body {
                Ok(text) => {
                    let mut bookname = String::new();
                    let mut reg = Regex::new("book_name\" content=\"[^\"]+").unwrap();
                    let name = reg.captures(text.as_str());
                    match name {
                        Some(name) => {
                            println!("Name: {}", &name[0][20..]);
                            name[0][20..].clone_into(&mut bookname);
                        }
                        None => {
                            println!("No name error");
                        }
                    }
                    let mut file = match File::create(format!("{}.txt", bookname)) {
                        Ok(file) => file,
                        Err(e) => {
                            println!("Error: {}", e);
                            return;
                        }
                    };
                    let mut chapter_list: Vec<String> = Vec::new();
                    let book = url.clone().replace("https://www.bqgui.cc", "");
                    let re_string = format!("href =\"{}[^\"]+", book);
                    reg = Regex::new(&re_string).unwrap();
                    let chapters = reg.captures_iter(text.as_str());
                    for chapter in chapters {
                        let chapter = format!("https://www.bqgui.cc{}", &chapter[0][7..]);
                        chapter_list.push(chapter.to_string());
                    }
                    println!("{:?}", chapter_list);
                    let mut count = Arc::new(tokio::sync::Semaphore::new(thread_number));
                    let mut chapter_content = Arc::new(Mutex::new(Vec::<(u32, String)>::new()));
                    let mut index = 0;
                    let mut join_list = Vec::new();
                    for chapter_url in chapter_list {
                        let url = chapter_url.clone();
                        let counter_clone = Arc::clone(&count);
                        let chapter_content_clone = Arc::clone(&chapter_content);
                        let permit = counter_clone.acquire_owned().await;
                        join_list.push(tokio::spawn(async move {
                            let content = get_content(&url).await;
                            match content {
                                Ok(content) => {
                                    let mut chapter_content = chapter_content_clone.lock().unwrap();
                                    println!("index: {}, url: {}", index, url);
                                    chapter_content.push((index, content));
                                }
                                Err(e) => {
                                    println!("Error: {}", e);
                                }
                            }
                            drop(permit);
                        }));
                        index += 1;
                    }

                    for join in join_list {
                        join.await.unwrap();
                    }
                    let mut chapter_content = chapter_content.lock().unwrap();
                    // 排序
                    chapter_content.sort_by(|a, b| a.0.cmp(&b.0));
                    for (_, content) in chapter_content.iter() {
                        file.write_all(content.as_bytes()).unwrap();
                    }
                    //system pause
                    let _ = std::process::Command::new("cmd")
                        .arg("/c")
                        .arg("pause")
                        .status();
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
