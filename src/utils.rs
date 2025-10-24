use ab_glyph::{Font,FontArc,GlyphId};
use std::process::{Command, Stdio};
// 引入编码检测和转换相关的库
use encoding_rs::*;
use encoding_rs::{UTF_8, GBK, UTF_16LE, UTF_16BE};
use std::fs::File;
use std::io::{Read};
use std::error::Error;

/// 支持多种编码的文本读取（无 chardetrs，用 encoding_rs 试探）
pub fn get_txt(file_name: &str) -> Result<String, Box<dyn Error>> {
    // 步骤1：读取文件原始字节（用于后续编码试探）
    let mut file = File::open(file_name)?;
    let mut raw_bytes = Vec::new();
    file.read_to_end(&mut raw_bytes)?;

    if raw_bytes.is_empty() {
        return Ok(String::new()); // 空文件直接返回空字符串
    }

    // 步骤2：定义常见编码优先级（按实际场景调整，中文优先试 GBK）
    let encodings = [
        ("UTF-8", UTF_8),
        ("GBK (中文)", GBK),
        ("UTF-16LE (Unicode小端)", UTF_16LE),
        ("UTF-16BE (Unicode大端)", UTF_16BE),
        ("Windows-1252 (英文常见)", WINDOWS_1252),
    ];

    // 步骤3：逐个尝试编码，返回第一个解码成功的结果
    for (enc_name, encoding) in encodings {
        // 尝试用当前编码解码（替换无法识别的字符，避免解码失败）
        let (decoded, _, had_errors) = encoding.decode(&raw_bytes);
        
        // 若无解码错误，直接返回结果；若有错误，尝试下一个编码
        if !had_errors {
            println!("自动识别编码：{}", enc_name);
            return Ok(decoded.into_owned());
        }
    }

    // 步骤4：所有编码尝试失败时，用 GBK 作为最终 fallback（中文场景优先）
    println!("所有编码试探失败，使用 GBK 作为最终 fallback");
    let (decoded, _, _) = GBK.decode(&raw_bytes);
    Ok(decoded.into_owned())
}

/// 检查文件夹是否存在，不存在则创建
/// 返回 Result<(), std::io::Error>，成功时返回 Ok(())，失败时返回错误信息
/* pub fn ensure_directory_exists(path: &str,flag:u8) -> Result<(), std::io::Error> {
    let dir_path = Path::new(path);    
    // 检查文件夹是否存在
    if dir_path.exists() {
        // 确认路径指向的是一个文件夹
        if dir_path.is_dir() {
            println!("文件夹已存在: {}", path);
            Ok(())
        } else {
            // 路径存在但不是文件夹
            Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("路径已存在但不是文件夹: {}", path)
            ))
        }
    } else {
        // flag为1时，文件夹不存在，创建它
        // 使用 create_dir_all 可以创建多级目录
        if flag == 1 {
            fs::create_dir_all(dir_path)?;
            println!("文件夹创建成功: {}", path);
            Ok(())
        }   
        else {
            Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("文件夹不存在: {}", path)
            ))
        }
    }
} */

/* pub fn delete_dir(path: &str) -> Result<(), std::io::Error> {
        // 替换你的删除逻辑
    match fs::remove_dir(path) {
        Ok(_) => {
            println!("{}目录删除成功", path);
            Ok(())
        },
        Err(e) => {          
            // 如果是目录非空的错误，尝试递归删除
            if e.kind() == io::ErrorKind::DirectoryNotEmpty {
                if let Err(ee) = fs::remove_dir_all(path) {
                    eprintln!("递归删除失败: {}", ee);
                    Err(ee)
                } else {
                    ("{}目录删除成功", path);
                    Ok(())  
                }
            }
            else {
                Err(e)
            }
        }
    }
} */


pub fn char_is_exist(font: &FontArc, c: char) -> bool {
    // Get a glyph for 'q' with a scale & position.
    let glyph_id = font.glyph_id(c);
    if glyph_id == GlyphId(0) {
        false
    }else{
        true    
    }
}


/// 过滤行集合中的空白行
fn filter_blank_lines(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        // 只保留非空白行（trim后不为空）
        .filter(|line| !line.trim().is_empty())
        // 克隆字符串以获取所有权
        .cloned()
        .collect()
}
pub fn split_into_lines(text: &str, chars_per_line: usize) -> Vec<String> {
    // 存储所有行的集合
    let mut lines = Vec::new();
    // 存储当前正在构建的行
    let mut current_line = String::new();
    
    let chars: Vec<char> = text.chars().collect();

   // 记录当前行已添加的字符数
    let mut char_count = 0;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i]; 
            
        // 处理换行符
        if c == '\n' {
            if !current_line.trim().is_empty() {
                lines.push(current_line);
            }
            current_line = String::new();
            char_count = 0;
            i += 1;
            continue;
        }

        // 添加当前字符到行
        current_line.push(replace_char(c));        
        // 非标点符号计入字符计数
        if is_punctuation(c) != 1 {               
            char_count += 1;
        }

        // 当字符数达到指定数量时检查是否需要包含下一个标点
        if char_count == chars_per_line {
            // 检查是否有下一个字符且为标点
            if i + 1 < chars.len() && (is_punctuation(chars[i + 1]) == 1) {
                // 加入下一个标点
                let char_tmp = replace_char(chars[i + 1]);
                current_line.push(char_tmp);                
                i += 1; // 跳过已处理的标点
            }            
            // 完成当前行
            lines.push(current_line);
            current_line = String::new();
            char_count = 0;
        }
        i += 1;
    }
    // 处理最后一行（如果存在未完成的行）
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    // 过滤空白行
    filter_blank_lines(&lines)
}

/// 将行集合按指定行数分割成多页
/// 参数:
/// - lines: 行集合
/// - lines_per_page: 每页包含的行数
/// 返回: 分页后的页集合，每个元素是一页（包含多行）
pub fn split_into_pages(lines: &[String], lines_per_page: usize) -> Vec<Vec<String>> {
    // 存储所有页的集合
    let mut pages = Vec::new();
    // 存储当前正在构建的页，预分配容量提升性能
    let mut current_page = Vec::with_capacity(lines_per_page);
    // 遍历每一行
    for line in lines {
        // 将当前行添加到当前页（克隆字符串以获得所有权）
        current_page.push(line.clone());
        // 当行数达到每页指定数量时，完成当前页
        if current_page.len() == lines_per_page {
            // 将当前页添加到页集合中
            pages.push(current_page);
            // 重置当前页，准备下一页
            current_page = Vec::with_capacity(lines_per_page);
        }
    }
    // 处理最后一页（如果存在未完成的页）
    if !current_page.is_empty() {
        pages.push(current_page);
    }
    pages
}

pub fn replace_char(c: char) -> char {
    match c {
        '1' => '一',
        '2' => '二',
        '3' => '三',
        '4' => '四',
        '5' => '五',
        '6' => '六',
        '7' => '七',
        '8' => '八',
        '9' => '九',
        '0' => '〇',
        '@' => ' ',
        '\r' => ' ',
        '\t' => ' ',
        '“' => '『',
        '”' => '』',
        '‘' => '「',
        '’' => '」',
        _ => c,
    }
}

pub fn is_punctuation(c: char) -> u8 {
    let punctuation_chars = "，@。！？、；：•□";
    let no_read_chars = "〇";
    let punctuation_chars_rotate = "{}（）……<>【】《》「」『』";
    if no_read_chars.contains(c) {
        // 是无读字符
        0
    } else if punctuation_chars.contains(c) {
        // 是标点符号
        1
    } else if punctuation_chars_rotate.contains(c) {
        // 是需要旋转的标点符号
        3
    } else {
        // 不是标点符号，也不是无读字符
        2
    }
}

/// 检查Ghostscript是否已安装
pub fn is_ghostscript_installed() -> bool {
    // 尝试运行`gs --version`命令，成功执行则说明已安装
    match Command::new("gswin64c")
        .arg("--version")
        .stdout(Stdio::null())  // 忽略输出
        .stderr(Stdio::null())
        .status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}


pub fn pdf_compress(input_pdf: &str, output_pdf: &str, quality: u8) -> Result<(), Box<dyn std::error::Error>> {
    // 构建Ghostscript命令参数
    // 确保质量参数在有效范围
    let pdf_settings = match quality {
        0..=30 => "/screen",    // 高压缩（低质量）
        31..=70 => "/ebook",    // 中等压缩
        _ => "/printer"         // 低压缩（高质量）
    };
    let command = format!(
        "gswin64c -sDEVICE=pdfwrite -dPDFSETTINGS={} -dJPEGQ={} -dNOPAUSE -dQUIET -dBATCH -dAutoRotatePages=/None -sOutputFile={} {}",
        pdf_settings,
        quality,
        output_pdf,
        input_pdf
    );
    println!("{}", command);
    let output = Command::new("cmd")
        .arg("/c")
        .arg(command)
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        println!("成功压缩PDF: {}", output_pdf);
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(format!("压缩失败: {}", error_msg).into())
    }
     
}

