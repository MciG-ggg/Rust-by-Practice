use std::error::Error;
use std::fs;
use std::env;

// use docx_rust::document::Paragraph;
// use docx_rust::DocxFile;
use std::fs::File;
use xml::reader::{EventReader, XmlEvent};
use std::io::Read;
use zip::ZipArchive;

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub extension: String,
}

impl Config {
    /*
    初始化Config
    */
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // if args.len() < 3 {
        //     return Err("NOT ENOUGH ARGUMENTS");
        // }

        // let query = args[1].clone();
        // let file_path = args[2].clone(); 

        // 第一个参数是程序名，由于无需使用，因此这里直接空调用一次
        args.next();
        
        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string")
        };
        
        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };

        //获取文件扩展名
        
        let extension = get_extension(&file_path).unwrap();
        
        // let ignore_case = env::var("IGNORE_CASE").is_ok(); 
        // let ignore_case = env::var("IGNORE_CASE").map_or(false, |var| var.eq("1"));
        /*
            同时使用命令行参数和环境变量的方式来控制大小写不敏感，
            其中环境变量的优先级更高，
            也就是两个都设置的情况下，
            优先使用环境变量的设置。
         */

        let ignore_case = match env::var("IGNORE_CASE") {
            Ok(val) => val == "1",
            Err(_) => {
                let mut flag = false;
                args.into_iter().for_each(|arg| {
                    if arg == "-i" || arg == "--ignore-case" {
                        flag = true;
                    }
                });

                flag
            }
        };

        Ok(Config { 
            query,
            file_path,
            ignore_case,
            extension,
        })
    }

}

/*
    执行minigrep
*/
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    
    let mut contents = String::new();
    let mut text = String::new();

    let results = match &*config.extension{
        "txt" => {
            contents = fs::read_to_string(&config.file_path)?;
            if config.ignore_case {
                Ok(search_case_insensitive(&config.query, &contents))
            } else {
                Ok(search(&config.query, &contents))
            }
        }
        "docx" => {
            let file = File::open(&config.file_path)?;
            
            let mut archive = ZipArchive::new(file)?;
            

            // 遍历 ZIP 文件中的所有条目
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let file_name = file.name();

                // 找到 document.xml 文件
                if file_name == "word/document.xml" {
                    println!("Reading: {}", file_name);

                    // 读取文件内容
                    file.read_to_string(&mut contents)?;
                    text = extract_text_from_xml(&contents).unwrap();
                    
                }
            }
            if config.ignore_case {
                Ok(search_case_insensitive(&config.query, &text))
            } else {
                Ok(search(&config.query, &text))
            }
        }
        _ => Err("Unsupported extension"),
    };

    match results {
        Ok(lines) => {
            for line in lines {
                println!("{:?}", line);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))
        }
    }

}
// 提取 XML 中的文本内容，并按段落划分
fn extract_text_from_xml(xml: &str) -> Result<String, Box<dyn Error>> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let parser = EventReader::from_str(xml);
    let mut in_paragraph = false;

    for event in parser {
        match event? {
            XmlEvent::StartElement { name, .. } => {
                if name.local_name == "p" {
                    in_paragraph = true;
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "p" && !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                    in_paragraph = false;
                }
            }
            XmlEvent::Characters(s) => {
                if in_paragraph {
                    current_line.push_str(&s);
                }
            }
            _ => {}
        }
    }

    // 添加最后一行，以防最后一个段落在文件末尾
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    let mut result = String::new();
    for line in lines {
        result.push_str(&line);
        result.push_str("\n");
    }

    Ok(result)
}
/*
    大小写敏感
    搜索search：
    看哪一行存在该字符串
*/
fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()    
}

/*
    大小写不敏感
    搜索search：
    看哪一行存在该字符串
*/
fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut results = Vec::new();
    
    for line in contents.lines(){
        if line.to_lowercase().contains(&query){
            results.push(line);        
        }
    }

    results
}
/*
    大小写敏感
    对docx文件
    search
*/
fn search_docx<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    

    results
}
/*
    大小写不敏感
    对docx文件
    search
*/
fn search_docx_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    vec![]
}
/*
    辅助函数
    获取文件扩展名
*/
fn get_extension(file_path: &str) -> Result<String, &str> {
    match file_path.rfind('.') {
        Some(pos) => Ok(String::from(&file_path[pos + 1..])),
        None => Err("No extension found"),
    }
}
/////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////

/*
    TEST
    ARE
    HERE
*/
#[cfg(test)]
mod tests {
    use super::*;
/*
    测试search的基本功能
*/
    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";
        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }
/*
    测试大小写不敏感功能是否实现
*/
    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search(query, contents)
        );
    }
/*
    测试后缀名读取是否成功
*/
    #[test]
    fn read_extension() {
        let file_path = "poem.txt";
        assert_eq!("txt".to_string(), get_extension(&file_path).unwrap());
        let file_path = "poem.docx";
        assert_eq!("docx".to_string(), get_extension(&file_path).unwrap());
    }

    
    #[test]
    fn case_docx() {
        
    }

    #[test]
    fn case_docx_insensitive() {

    }
}