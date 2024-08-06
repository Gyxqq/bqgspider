use regex::Regex;
use reqwest::{self, Url};
use std::{
    fs::File,
    io::{stdin, Write},
};
use tokio;
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
                    for chapter_url in chapter_list {
                        loop {
                            let body = reqwest::get(&chapter_url).await;
                            match body {
                                Ok(response) => {
                                    let body = response.text().await;
                                    match body {
                                        Ok(text) => {
                                            reg =
                                                Regex::new("<h1 class=\"wap_none\">[^<]+").unwrap();
                                            let title = reg.captures(text.as_str());
                                            match title {
                                                Some(title) => {
                                                    println!("Title: {}", &title[0][21..]);
                                                    file.write_all(
                                                        format!("{}\n", &title[0][21..]).as_bytes(),
                                                    )
                                                    .unwrap();
                                                }
                                                None => {
                                                    println!("No title error");
                                                }
                                            }
                                            reg = Regex::new("[^>]+<br /><br />").unwrap();
                                            let contents = reg.captures_iter(text.as_str());
                                            let mut content_string = String::new();
                                            for content in contents {
                                                // 跳过最后一个
                                                if content[0].contains("https://m.bqgui.cc") {
                                                    continue;
                                                }
                                                // println!("{}", &content[0]);
                                                content_string.push_str(
                                                    &content[0].replace("<br /><br />", "\n"),
                                                );
                                            }
                                            content_string.push_str("\n\n");
                                            file.write_all(content_string.as_bytes()).unwrap();
                                            break;
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
                        }
                    }
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
