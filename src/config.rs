use printpdf::{Color, Rgb,Point,Pt}; 
use serde::{Deserialize}; 
use sonic_rs::{from_str, to_string_pretty, Serialize};
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::{Write};

pub const MM_PER_INCH: f32 = 25.4;
pub const PT_PER_INCH: f32 = 72.0;
//pub const PT_TO_MM: f32 = MM_PER_INCH / PT_PER_INCH;
pub const MM_TO_PT: f32 = PT_PER_INCH / MM_PER_INCH;
pub const FONT_OFFSET_SCALE: f32 = 1.2;
pub const PUN_SCALE: f32 = 0.5; //非占位标点与正文字体大小比值
pub const PUN_PUB: f32 = 0.8;   //占位符号Y偏移比例

#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Parameter {
    pub pageinfo: Pager,
    pub canvas: Canvas,
    pub tail: Tail,
    pub color: DrawColor,
    pub font: FontSet,
    pub book: Book,
    pub file: FileInfo,
    pub content: Content,
    pub pagination: Pagination,
    pub title: Title,
}


#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Book {
    pub name: String,
    pub author: String,
    pub creater: String,
}
#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct FontSet {
    pub main_path: String,
    pub backup_path: String,
}

#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Pager {
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub page_width_pt: Pt,
    pub page_height_pt: Pt,
    pub page_top_margin_pt: Pt,
    pub page_bottom_margin_pt: Pt,
    pub page_left_margin_pt: Pt,
    pub page_right_margin_pt: Pt,
    pub column_count: usize,
    pub center_width_pt: Pt,
    pub tail_margin_pt: Pt,
    pub tail_space_pt: Pt,
    pub tail_long_offset_pt: Pt,
    pub tail_short_offset_pt: Pt,
}

#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Template {
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub bgcolor: String,
    pub linecolor: String,
    pub tail: Tail,
    pub canvas: Canvas,
}

#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct DrawColor {
    pub bg: String,
    pub line: String,
    pub draw: String,
}
#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Canvas {
    pub point_left_bottom: Point,
    pub point_right_top: Point,
    pub point_left_top: Point,
    pub point_right_bottom: Point,

    pub point_center_left_bottom: Point,
    pub point_center_right_bottom: Point,
    pub point_center_left_top: Point,
    pub point_center_right_top: Point,

    pub width_pt: Pt,
    pub height_pt: Pt,
    pub column_width_pt: Pt,
    pub line_width_pt: Pt,
    pub line_offset_pt: Pt,
}

#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Tail {
    pub point_up_left_bottom: Point,
    pub point_up_left_top: Point,
    pub point_up_right_bottom: Point,
    pub point_up_right_top: Point,
    pub point_down_left_bottom: Point,
    pub point_down_left_top: Point,
    pub point_down_right_bottom: Point,
    pub point_down_right_top: Point,
    pub point_up_center: Point,
    pub point_down_center: Point,   
    pub point_line_up_left: Point,
    pub point_line_up_right: Point,
    pub point_line_down_left: Point, 
    pub point_line_down_right: Point, 
}

#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct FileInfo {
    pub inputpath: String,
    pub outputpath: String,
    pub compressratio:u8,
}

#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Title {
    pub loc_start_x_pt: Pt,            // 标题开始x坐标
    pub loc_start_y_pt: Pt,            // 标题开始y坐标
    pub title_offset: Pt,              // 标题偏移量
    pub space_x_pt: Pt,                // 横向位移
    pub space_y_pt: Pt,                // 纵向间距
    pub max_chars: i32,                 // 最大字符数
    pub font_size_pt: f32,              // 字体大小
    pub font_color: String,                // 字体颜色
}

#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Pagination {
    pub loc_start_x_pt: Pt,            // 分页开始x坐标
    pub loc_start_y_pt: Pt,            // 分页开始y坐标
    pub font_size_pt: f32,              // 字体大小
    pub font_color: String,                // 字体颜色
}
#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Content {
    pub loc_start_x1_pt: Pt,            // 每页开始x坐标
    pub loc_start_y1_pt: Pt,            // 每页开始y坐标
    pub loc_start_x2_pt: Pt,            // 每页开始x坐标
    pub loc_start_y2_pt: Pt,            // 每页开始y坐标    
    pub content_offset: Pt,            // 内容偏移量
    pub space_x_pt: Pt,                // 横向位移
    pub space_y_pt: Pt,                // 纵向间距
    pub max_chars: i32,                 // 最大字符数
    pub font_size_pt: f32,              // 字体大小
    pub pun_font_size_pt: f32,          // 标点大小
    pub font_color: String,                // 字体颜色
}
pub fn default(base:Base) -> Parameter {

    let book = Book {
            name: base.bookname,
            author: base.bookauthor,
            creater: base.pdfcreater,
        };
    let file = FileInfo {
            inputpath: base.bookinputpath,
            outputpath: base.bookoutputpath,
            compressratio: base.compressratio,
        };

    let font = FontSet {
        main_path: base.main_font_path,
        backup_path: base.backup_font_path,
    };
    
    let page = Pager {
        page_width_mm: base.page_width_mm,
        page_height_mm: base.page_height_mm,
        page_width_pt: Pt(base.page_width_mm * MM_TO_PT),
        page_height_pt: Pt(base.page_height_mm * MM_TO_PT),
        page_top_margin_pt: Pt(base.page_top_margin_mm * MM_TO_PT),
        page_bottom_margin_pt: Pt(base.page_bottom_margin_mm * MM_TO_PT),
        page_left_margin_pt: Pt(base.page_left_margin_mm * MM_TO_PT),
        page_right_margin_pt: Pt(base.page_right_margin_mm * MM_TO_PT),
        column_count: base.column_count,
        center_width_pt: Pt(base.center_width_mm * MM_TO_PT),
        tail_margin_pt: Pt(base.tail_margin_mm * MM_TO_PT),
        tail_space_pt: Pt(base.tail_space_mm * MM_TO_PT),
        tail_long_offset_pt: Pt(base.tail_long_offset_mm * MM_TO_PT),
        tail_short_offset_pt: Pt(base.tail_short_offset_mm * MM_TO_PT),
    };
    
    let canvas = Canvas {
        line_width_pt: Pt(base.line_width_pt),
        line_offset_pt: Pt(base.line_offset_pt),

        width_pt: page.page_width_pt 
                - page.page_left_margin_pt 
                - page.page_right_margin_pt,

        height_pt: page.page_height_pt 
                - page.page_top_margin_pt 
                - page.page_bottom_margin_pt,

        column_width_pt: (page.page_width_pt 
                - page.page_left_margin_pt 
                - page.page_right_margin_pt
                - page.center_width_pt) / page.column_count as f32,

        point_left_bottom: Point{
            x: page.page_left_margin_pt, 
            y: page.page_bottom_margin_pt,
        },
        point_right_top: Point{
            x: page.page_width_pt - page.page_right_margin_pt, 
            y: page.page_height_pt - page.page_top_margin_pt,
        },
        point_left_top: Point{
            x: page.page_left_margin_pt, 
            y: page.page_height_pt - page.page_top_margin_pt,
        },
        point_right_bottom: Point{
            x: page.page_width_pt - page.page_right_margin_pt, 
            y: page.page_bottom_margin_pt,
        },
        point_center_left_bottom: Point{
            x: (page.page_width_pt - page.center_width_pt) / 2.0, 
            y: page.page_bottom_margin_pt,
        },
        point_center_right_bottom: Point{
            x: (page.page_width_pt + page.center_width_pt) / 2.0, 
            y: page.page_bottom_margin_pt,
        },
        point_center_left_top: Point{
            x: (page.page_width_pt - page.center_width_pt) / 2.0, 
            y: page.page_height_pt - page.page_top_margin_pt,
        },
        point_center_right_top: Point{
            x: (page.page_width_pt + page.center_width_pt) / 2.0, 
            y: page.page_height_pt - page.page_top_margin_pt,
        },
    };

    let tail = Tail {        
        point_up_left_bottom: Point{
            x: (page.page_width_pt - page.center_width_pt) / 2.0, 
            y: page.page_height_pt - page.page_top_margin_pt
             - page.tail_margin_pt - page.tail_long_offset_pt,
        },
        point_up_left_top: Point{
            x: (page.page_width_pt - page.center_width_pt) / 2.0, 
            y: page.page_height_pt - page.page_top_margin_pt - page.tail_margin_pt,
        },
        point_up_right_bottom: Point{
            x: (page.page_width_pt + page.center_width_pt) / 2.0, 
            y: page.page_height_pt - page.page_top_margin_pt
             - page.tail_margin_pt - page.tail_long_offset_pt,
        },
        point_up_right_top: Point{
            x: (page.page_width_pt + page.center_width_pt) / 2.0, 
            y: page.page_height_pt - page.page_top_margin_pt - page.tail_margin_pt,
        },
        point_up_center: Point {
            x:(page.page_width_pt) / 2.0, 
            y:(page.page_height_pt - page.page_top_margin_pt
             - page.tail_margin_pt- page.tail_short_offset_pt),
            },

        point_down_left_bottom: Point{
            x: (page.page_width_pt - page.center_width_pt) / 2.0, 
            y: page.page_bottom_margin_pt 
                + page.tail_margin_pt,
        },
        point_down_left_top: Point{
            x: (page.page_width_pt - page.center_width_pt) / 2.0, 
            y: page.page_bottom_margin_pt + page.tail_margin_pt + page.tail_long_offset_pt,
        },
        point_down_right_bottom: Point{
            x: (page.page_width_pt + page.center_width_pt) / 2.0, 
            y: page.page_bottom_margin_pt + page.tail_margin_pt,
        },
        point_down_right_top: Point{
            x: (page.page_width_pt + page.center_width_pt) / 2.0, 
            y: page.page_bottom_margin_pt  + page.tail_margin_pt + page.tail_long_offset_pt,
        },        
        point_down_center: Point{
            x:(page.page_width_pt) / 2.0, 
            y:(page.page_bottom_margin_pt  + page.tail_margin_pt + page.tail_short_offset_pt),
        },
        
        point_line_up_left: Point{
            x: (page.page_width_pt - page.center_width_pt) / 2.0, 
            y: page.page_height_pt - page.page_top_margin_pt
             - page.tail_margin_pt + page.tail_space_pt,
        },
        point_line_up_right: Point {
            x: (page.page_width_pt + page.center_width_pt) / 2.0, 
            y: page.page_height_pt - page.page_top_margin_pt
             - page.tail_margin_pt + page.tail_space_pt,
        },
        point_line_down_left: Point{
            x: (page.page_width_pt - page.center_width_pt) / 2.0, 
            y: page.page_bottom_margin_pt + page.tail_margin_pt - page.tail_space_pt,
        },
        point_line_down_right: Point{
            x: (page.page_width_pt + page.center_width_pt) / 2.0, 
            y: page.page_bottom_margin_pt + page.tail_margin_pt - page.tail_space_pt,
        },
    };

    let color = DrawColor {
        bg: base.bg_color.clone(),
        line: base.line_color.clone(),
        draw: base.draw_color.clone(),
    };

    let content_size = Pt(base.content_font_size_pt); 
    let content_offset = (canvas.column_width_pt - content_size) / 2.0;

    let content = Content{
        content_offset : content_offset,
        loc_start_x1_pt: canvas.point_right_top.x - (canvas.column_width_pt + Pt(base.content_font_size_pt))/ 2.0,
        loc_start_y1_pt: canvas.point_right_top.y - content_size * FONT_OFFSET_SCALE,
        loc_start_x2_pt: canvas.point_center_left_top.x - (canvas.column_width_pt + Pt(base.content_font_size_pt))/ 2.0,
        loc_start_y2_pt: canvas.point_center_left_top.y - content_size * FONT_OFFSET_SCALE,
        space_x_pt: Pt(- canvas.column_width_pt.0),
        space_y_pt: Pt(- content_size.0 * FONT_OFFSET_SCALE),
        max_chars: (canvas.height_pt.0 / (content_size.0 * FONT_OFFSET_SCALE) )as i32,
        font_size_pt: base.content_font_size_pt,
        pun_font_size_pt: base.content_font_size_pt * PUN_SCALE,
        font_color: base.draw_color.clone(),
    };

    let title_size = Pt(base.title_font_size_pt); 
    let title_offset =  (page.center_width_pt - title_size) / 2.0;
    let title = Title{
        title_offset: title_offset,
        loc_start_x_pt: page.page_width_pt / 2.0 - Pt(title_size.0 / 2.0),
        loc_start_y_pt: tail.point_up_left_bottom.y - Pt(title_size.0 * FONT_OFFSET_SCALE),
        space_x_pt: Pt(0.0),
        space_y_pt: Pt(- title_size.0 * FONT_OFFSET_SCALE),
        max_chars: ((tail.point_up_left_bottom.y.0 - tail.point_down_left_top.y.0 
                  - page.center_width_pt.0)
                / (title_size.0 * FONT_OFFSET_SCALE) ) as i32,
        font_size_pt: base.title_font_size_pt,
        font_color: base.draw_color.clone(),        
    };
    
    let pagination = Pagination{
        loc_start_x_pt: page.page_width_pt / 2.0,
        loc_start_y_pt: tail.point_down_left_top.y + page.center_width_pt / 2.0,
        font_size_pt: base.content_font_size_pt * PUN_SCALE,
        font_color: base.draw_color.clone(),        
    };
    let param = Parameter {
        pageinfo: page,
        canvas: canvas,
        tail: tail,
        color: color,
        font: font,
        book: book,
        file: file,
        content: content,
        pagination: pagination,
        title: title,
    };    
    param
}
/* pub fn save_json(param: &Parameter,json_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json_str = to_string_pretty(&param)?;
    
    // 写入文件
    let mut file = File::create(json_path)?;
    file.write_all(json_str.as_bytes())?;    
    println!("数据已通过 sonic_rs 保存到 {}",json_path);
    Ok(())
} */
// 从JSON文件读取并解析为Base实例
/* pub fn load_config(config_path: &str) -> Result<Parameter, Box<dyn std::error::Error>> {
    // 定义JSON文件路径
    let json_path = Path::new(config_path);    
    // 检查文件是否存在
    if !json_path.exists() {
        eprintln!("JSON文件不存在: {}", json_path.display());
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("JSON文件不存在: {}", json_path.display()),
        )));
    }
    
    // 读取文件内容
    let mut file = File::open(json_path)?;
    let mut json_str = String::new();
    file.read_to_string(&mut json_str)?;    
    // 反序列化JSON到容器结构体
    let param: Parameter = from_str(&json_str)?;
    Ok(param)
} */
pub fn color_to_rgb(color: &str) -> Color {    
    match color {
        "白" => Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)),
        "黑" => Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
        "红" => Color::Rgb(Rgb::new(233.0/255.0, 49.0/255.0, 61.0/255.0, None)),
        "绿" => Color::Rgb(Rgb::new(0.0, 1.0, 0.0, None)),
        "蓝" => Color::Rgb(Rgb::new(14.0/255.0, 102.0/255.0, 150.0/255.0, None)),
        "灰黄" => Color::Rgb(Rgb::new(230.0/255.0, 224.0/255.0, 209.0/255.0, None)),
        "旧书" => Color::Rgb(Rgb::new(212.0/255.0, 184.0/255.0, 134.0/255.0, None)),
        "泛黄" => Color::Rgb(Rgb::new(245.0/255.0, 230.0/255.0, 196.0/255.0, None)),
        "浅黄" => Color::Rgb(Rgb::new(245.0/255.0, 240.0/255.0, 225.0/255.0, None)),
        "深黄" => Color::Rgb(Rgb::new(194.0/255.0, 166.0/255.0, 113.0/255.0, None)),
        "褐" => Color::Rgb(Rgb::new(74.0/255.0, 63.0/255.0, 53.0/255.0, None)),
        "深灰" => Color::Rgb(Rgb::new(58.0/255.0, 58.0/255.0, 58.0/255.0, None)),
        "墨" => Color::Rgb(Rgb::new(35.0/255.0, 35.0/255.0, 35.0/255.0, None)),
        _ => Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
    }
}

// 定义Base结构体（与JSON字段对应）
#[derive(Debug, Clone,Deserialize ,Serialize)]
pub struct Base {
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub page_top_margin_mm: f32,
    pub page_bottom_margin_mm: f32,
    pub page_left_margin_mm: f32,
    pub page_right_margin_mm: f32,
    pub column_count: usize,
    pub center_width_mm: f32,
    pub tail_margin_mm: f32,
    pub tail_space_mm: f32,
    pub tail_long_offset_mm: f32,
    pub tail_short_offset_mm: f32,
    pub bg_color: String,
    pub line_color: String,
    pub draw_color: String,
    pub content_font_size_pt: f32,
    pub title_font_size_pt: f32,
    pub number_font_size_scale: f32,
    pub punc_font_size_scale: f32,
    pub main_font_path: String,
    pub backup_font_path: String,
    pub bookname: String,
    pub bookauthor: String,
    pub pdfcreater: String,
    pub bookinputpath: String,
    pub bookoutputpath: String,
    pub line_width_pt: f32,
    pub line_offset_pt: f32,
    pub compressratio:u8,
}
// 实现从JSON解析到Base的逻辑
impl Base {
    pub fn default() -> Self {
        Self {
        page_width_mm: 297.0,                   // 页面宽度（毫米）
        page_height_mm: 210.0,                  // 页面高度（毫米）
        page_top_margin_mm: 20.0,               // 页面顶部边距（毫米）
        page_bottom_margin_mm: 8.0,             // 页面底部边距（毫米）
        page_left_margin_mm: 8.0,               // 页面左侧边距（毫米）
        page_right_margin_mm: 8.0,              // 页面右侧边距（毫米）
        column_count: 24,                       // 列数
        center_width_mm: 20.0,                  // 心页宽度（毫米）
        tail_margin_mm: 30.0,                   // 鱼尾边距（毫米）
        tail_space_mm: 0.5,                     // 鱼尾细线偏差（毫米）
        tail_long_offset_mm: 12.0,              // 鱼尾长端偏移（毫米）
        tail_short_offset_mm: 8.0,              // 鱼尾短端偏移（毫米）
        line_width_pt: 4.0,                     // 粗线宽（点）
        line_offset_pt: 4.5,                    // 粗线框偏移（点）
        bg_color: "泛黄".to_string(),             // 背景颜色
        line_color: "红".to_string(),           // 线颜色
        draw_color: "黑".to_string(),           // 绘制颜色
        content_font_size_pt: 18.0,             // 内容字体大小（点）
        title_font_size_pt: 24.0,              // 标题字体大小（点）
        number_font_size_scale: 0.4,           // 页码字体大小缩放比例
        punc_font_size_scale: 0.5,             // 标点符号字体大小缩放比例
        main_font_path: "./fonts/XiaolaiMonoSC-Regular.ttf".to_string(),      // 主字体路径
        backup_font_path: "./fonts/simfang-lite.ttf".to_string(),        // 备用字体路径
        bookname: "庄子".to_string(),                           // 书籍名称
        bookauthor: "庄子".to_string(),                         // 书籍作者
        pdfcreater: "测试创建人".to_string(),                    // PDF创建人
        bookinputpath: "./text/001.txt".to_string(),           // 书籍输入路径
        bookoutputpath: "./pdf/庄子.pdf".to_string(),           // 书籍输出路径 
        compressratio: 50,
        }
    }
    // 从JSON文件读取并解析为Base实例
    pub fn from_json_file(config_path: &str) -> Self {
        // 尝试读取文件
        let json_content = match fs::read_to_string(Path::new(config_path)) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("⚠️ 文件读取失败：{}，将使用默认配置", e);
                return Self::default(); // 返回默认值
            }
        };
        // 尝试解析JSON
        match from_str(&json_content) {
            Ok(base) => base,
            Err(e) => {
                eprintln!("⚠️ JSON解析失败：{}，将使用默认配置", e);
                Self::default() // 返回默认值
            }
        }    
    }
    pub fn save_json(base: &Base,json_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json_str = to_string_pretty(&base)?;    
    // 写入文件
    let mut file = File::create(json_path)?;
    file.write_all(json_str.as_bytes())?;    
    println!("数据已通过 sonic_rs 保存到 {}",json_path);
    Ok(())
}
}

