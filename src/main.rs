mod utils;
mod pdfoption;
mod config;
use std::path::{Path,PathBuf};
use config::*;
slint::include_modules!();

fn main() {    
    let ui = MainWindow::new().unwrap();
    //let (page,draw_color,file_info) = update(&ui);
    // 设置当前路径
    let dir_str = std::env::current_dir().unwrap().to_string_lossy().to_string();
    let _= ui.set_current_path(dir_str.clone().into());
    let ui_weak = ui.as_weak();
    //创建PDF
    let ui_weak_pdf = ui_weak.clone();
    ui.on_create_pdf(move || {
        // 在闭包中使用弱引用访问UI
        if let Some(ui) = ui_weak_pdf.upgrade() { 
            create_pdf(&ui);
        }
    });

    let ui_weak_pdf_compress: slint::Weak<MainWindow> = ui_weak.clone();
    // 压缩PDF
    ui.on_pdf_compress_gs(move || {
        // 在闭包中使用弱引用访问UI
        if let Some(ui) = ui_weak_pdf_compress.upgrade() { 
            pdf_compress_gs(&ui);
        }
    }) ;
    let ui_weak_config: slint::Weak<MainWindow> = ui_weak.clone();
    // 加载配置
    ui.on_get_config(move || {
        // 在闭包中使用弱引用访问UI
        if let Some(ui) = ui_weak_config.upgrade() { 
            get_config(&ui);
        }
    }) ;

    let ui_weak_save: slint::Weak<MainWindow> = ui_weak.clone();
    // 保存配置
    ui.on_save_config(move || {
        // 在闭包中使用弱引用访问UI
        if let Some(ui) = ui_weak_save.upgrade() { 
            save_config(&ui);
        }
    }) ;

    let ui_weak_template: slint::Weak<MainWindow> = ui_weak.clone();
    // 生成模板
    ui.on_make_template(move || {
        // 在闭包中使用弱引用访问UI
        if let Some(ui) = ui_weak_template.upgrade() { 
            create_template(&ui);
        }
    }) ;
    ui.run().unwrap();
}

fn create_template(ui: &MainWindow) {    
    let (page,draw_color,file_info) = update(&ui); 
    pdfoption::create_pdf_template(&page, &draw_color, &file_info);
    println!("创建模板成功");
    let _ = ui.set_outtext_config("创建模板成功".to_string().into());
}
fn create_pdf(ui: &MainWindow) {
    // 1. 获取字体路径并转换为 PathBuf（更安全的路径类型）
    let font_path = PathBuf::from(ui.get_font_path().as_str());
    let font_backup_path = PathBuf::from(ui.get_font_backup_path().as_str());

    // 2. 检查主字体是否存在且为文件（而非目录）
    let main_font_exists = font_path.is_file();
    // 检查备份字体是否存在且为文件
    let backup_font_exists = font_backup_path.is_file();
    if !main_font_exists || !backup_font_exists {
        println!("字体文件不存在: {}，或备份字体文件不存在: {}", font_path.display(), font_backup_path.display());
        let _ = ui.set_outtext(format!("字体文件不存在: {}，或备份字体文件不存在: {}", font_path.display(), font_backup_path.display()).into());
        return;
    }
    if !PathBuf::from(ui.get_input_path().as_str()).exists() {
        println!("输入文件不存在: {}", ui.get_input_path().as_str());
        let _ = ui.set_outtext(format!("输入文件不存在: {}", ui.get_input_path().as_str()).into());
        return;
    }
    let (page,draw_color,file_info) = update(&ui); 
    pdfoption::create_pdf(&page, &draw_color, &file_info);
    println!("创建{}成功", ui.get_output_path());
    let _ = ui.set_outtext(format!("创建{}成功", ui.get_output_path()).into());
}

fn pdf_compress_gs(ui: &MainWindow) {
    let pdf_path = ui.get_output_path();
            let compress_ratio = ui.get_compress_ratio() as u8;
            println!("需要压缩的PDF路径: {}", pdf_path);
            if !utils::is_ghostscript_installed() {
                println!("未检测到Ghostscript安装。请先安装Ghostscript并确保gs命令在环境变量中。");
                let _ = ui.set_outtext(format!("未检测到Ghostscript安装。请先安装Ghostscript并确保gs命令在环境变量中。").into());
            }
            else {
                if let Err(e) = utils::pdf_compress(
                    pdf_path.as_str(),
                    pdf_path.replace(".pdf", "_compressed.pdf").as_str(),
                    compress_ratio) {
                    println!("压缩{}失败: {:?}", pdf_path, e);
                    let _ = ui.set_outtext(format!("压缩{}失败: {:?}", pdf_path, e).into());
                }
                else {
                    println!("压缩{}成功", pdf_path);
                    let _ = ui.set_outtext(format!("压缩{}成功", pdf_path).into());
                }
            }
}

fn get_config(ui: &MainWindow)  {
    let config_path = ui.get_config_path();
    println!("需要加载的配置文件路径: {}", config_path);
    let _ = ui.set_outtext(format!("需要加载的配置文件路径: {}", config_path).into());
    if !PathBuf::from(config_path.as_str()).exists() {
        println!("配置文件不存在: {}", config_path.as_str());
        let _ = ui.set_outtext(format!("配置文件不存在: {}", config_path.as_str()).into());
        return;
    }
    let (page, draw_color, file_info) = load_config(config_path.as_str());  
    //combox.set_selected_index(base.font.main_index as u32);
    let _ = update_ui(&ui,&page, &draw_color, &file_info);
    println!("加载的配置文件完成: {}", config_path);
    let _ = ui.set_outtext_config(format!("加载的配置文件完成: {}", config_path).into());
}

fn save_config(ui: &MainWindow) {
    let config_path = ui.get_config_path();
    println!("需要保存的配置文件路径: {}", config_path);
    let _ = ui.set_outtext(format!("需要保存的配置文件路径: {}", config_path).into());
    // 提取文件所在的文件夹（父目录）
    if let Some(parent_dir) = Path::new(config_path.as_str()).parent() {
        // 检查父目录是否存在
        if !parent_dir.exists() {
            println!("文件所在的文件夹不存在: {}", parent_dir.display());
            let _ = ui.set_outtext(format!("文件所在的文件夹不存在: {}", parent_dir.display()).into());
            return; // 或返回错误
        } else {
            println!("文件夹存在: {}", parent_dir.display());
            let _ = ui.set_outtext(format!("文件夹存在: {}", parent_dir.display()).into());
        }
        } else {
            // 处理无法获取父目录的情况（如路径是根目录）
            println!("无法获取文件所在的文件夹路径");
            let _ = ui.set_outtext(format!("无法获取文件所在的文件夹路径").into());
            return;
        }
    if let Err(e) = save_config_file(ui,config_path.as_str()) {
        println!("保存配置文件失败: {:?}", e);
        let _ = ui.set_outtext(format!("保存配置文件失败: {:?}", e).into());
        return;
    } 
    println!("配置文件已保存: {}", config_path);
    let _ = ui.set_outtext_config(format!("配置文件已保存: {}", config_path).into()); 
}

fn load_config(path: &str) -> (Pager,DrawColor,FileInfo) {
    let (page,drcolor,fileinfo) = config::from_json_file (path);
    (page,drcolor,fileinfo)
}

fn save_config_file(ui: &MainWindow,path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (page,draw_color,file_info) = update(&ui);
    config::save_json(page,draw_color,file_info,path)?;
    Ok(())
}


fn update(ui: &MainWindow) -> (Pager,DrawColor,FileInfo){
    let page_width_mm= ui.get_page_width_mm().parse::<f32>().unwrap();
    let page_height_mm =  ui.get_page_height_mm().parse::<f32>().unwrap();
    let column_count = ui.get_column_count() as usize;
    let center_width_mm = ui.get_center_width_mm() as f32;
    let page_top_margin_mm = ui.get_page_top_margin_mm() as f32;
    let page_left_margin_mm = ui.get_page_left_margin_mm() as f32;
    let tail_margin_mm = ui.get_tail_margin_mm() as f32;   
    let col_width_mm = (page_width_mm - center_width_mm - page_left_margin_mm * 2.0) 
                           / column_count as f32;
    
    let page_bottom_margin_mm = page_left_margin_mm;
    let page_right_margin_mm = page_left_margin_mm;   
    let tail_long_offset_mm = tail_margin_mm * 0.4;
    let tail_short_offset_mm = tail_margin_mm * 0.25;
    let title_font_size_pt = (center_width_mm * 0.45 * MM_TO_PT).round() as f32;
    let content_font_size_pt = (col_width_mm * 0.6 * MM_TO_PT).round() as f32;

    let bg_color = ui.get_background_color().to_string();
    let draw_color = ui.get_font_color().to_string();
    let line_color = ui.get_line_color().to_string();
    let bookname =  ui.get_book_name().to_string();
    let bookinputpath = ui.get_input_path().to_string();
    let bookoutputpath = ui.get_output_path().to_string();
    let main_font_path = ui.get_font_path().to_string();
    let backup_font_path = ui.get_font_backup_path().to_string();

    let compressratio = ui.get_compress_ratio() as u8;

    let page = Pager {
        page_width_mm:  page_width_mm,
        page_height_mm:page_height_mm,
        column_count: column_count,
        page_top_margin_mm: page_top_margin_mm,
        page_bottom_margin_mm: page_bottom_margin_mm,
        page_left_margin_mm: page_left_margin_mm,
        page_right_margin_mm: page_right_margin_mm,  
        center_width_mm:center_width_mm,     
        tail_margin_mm: tail_margin_mm,
        tail_space_mm: LINE_SPACE_MM,
        tail_long_offset_mm: tail_long_offset_mm,
        tail_short_offset_mm: tail_short_offset_mm,
        line_width_pt: LINE_WIDTH_PT,
        line_offset_pt: LINE_OFFSET_PT,
        title_font_size_pt: title_font_size_pt,
        content_font_size_pt: content_font_size_pt,
    };
    
    let draw_color = DrawColor{
        bg:bg_color,
        line:line_color,
        draw:draw_color,
    };

    let fileinfo = FileInfo{
        name:bookname.clone(),
        inputpath:bookinputpath,
        outputpath:bookoutputpath,
        author:bookname.clone(),
        creater:bookname.clone(),
        main_path:main_font_path,
        backup_path:backup_font_path,
        compressratio:compressratio,
    };
    (page,draw_color,fileinfo)
}
fn update_ui(ui: &MainWindow,page: &Pager, drawcolor: &DrawColor,fileinfo: &FileInfo) {
    ui.set_page_width_mm(page.page_width_mm.to_string().into());
    ui.set_page_height_mm(page.page_height_mm.to_string().into());
    ui.set_center_width_mm((page.center_width_mm as i32).into());
    ui.set_page_top_margin_mm((page.page_top_margin_mm as i32).into());
    ui.set_page_left_margin_mm((page.page_left_margin_mm as i32).into());
    ui.set_tail_margin_mm((page.tail_margin_mm as i32).into());
    ui.set_column_count((page.column_count as i32).into());
    ui.set_font_color(drawcolor.draw.clone().into());
    ui.set_background_color(drawcolor.bg.clone().into());
    ui.set_line_color(drawcolor.line.clone().into());
    ui.set_font_path(fileinfo.main_path.to_string().into());
    ui.set_font_backup_path(fileinfo.backup_path.to_string().into());
    ui.set_input_path(fileinfo.inputpath.to_string().into());
    ui.set_output_path(fileinfo.outputpath.to_string().into());
    ui.set_book_name(fileinfo.name.to_string().into());
    ui.set_compress_ratio(fileinfo.compressratio.into());
    let direction = if page.page_width_mm > page.page_height_mm {"横向"}else{"纵向"};
    ui.set_canvas_direction(direction.into());

    // 主逻辑
    if direction == "横向" {
        let paper_size = get_paper_size(page.page_width_mm);
        ui.set_canvas_size(paper_size.into());
    } else {
        let paper_size = get_paper_size(page.page_height_mm);
        ui.set_canvas_size(paper_size.into());
    }  
}

// 提取重复的匹配逻辑为函数
fn get_paper_size(mm: f32) -> &'static str {
    match mm {
        297.0 => "A4",
        250.0 => "B5",
        260.0 => "16K",
        184.0 => "32K",
        _ => "A4", // 默认A4
    }
}