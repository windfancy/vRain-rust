mod utils;
mod pdfoption;
mod config;
use std::path::{Path,PathBuf};
use config::*;
slint::include_modules!();

fn main() {    
    create();
}

fn create() {
   let ui = MainWindow::new().unwrap();
    let base = config::Base::default();
    update_ui(&ui,&base);
    // 设置当前路径
    let dir_str = std::env::current_dir().unwrap().to_string_lossy().to_string();
    let _= ui.set_current_path(dir_str.clone().into());
    let ui_weak = ui.as_weak();
    //创建PDF
    let ui_weak_pdf = ui_weak.clone();
    ui.on_create_pdf(move || {
        // 在闭包中使用弱引用访问UI
        if let Some(ui) = ui_weak_pdf.upgrade() { 
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
            let param = update_config(&ui); 
            pdfoption::create_pdf(&param);
            println!("创建{}成功", ui.get_output_path());
            let _ = ui.set_outtext(format!("创建{}成功", ui.get_output_path()).into());
        }
    });

    let ui_weak_pdf_compress: slint::Weak<MainWindow> = ui_weak.clone();
    // 压缩PDF
    ui.on_pdf_compress_gs(move || {
        // 在闭包中使用弱引用访问UI
        if let Some(ui) = ui_weak_pdf_compress.upgrade() { 
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
    }) ;
    let ui_weak_config: slint::Weak<MainWindow> = ui_weak.clone();
    // 加载配置
    ui.on_get_config(move || {
        // 在闭包中使用弱引用访问UI
        if let Some(ui) = ui_weak_config.upgrade() { 
            let config_path = ui.get_config_path();
            println!("需要加载的配置文件路径: {}", config_path);
            let _ = ui.set_outtext(format!("需要加载的配置文件路径: {}", config_path).into());
            if !PathBuf::from(config_path.as_str()).exists() {
                println!("配置文件不存在: {}", config_path.as_str());
                let _ = ui.set_outtext(format!("配置文件不存在: {}", config_path.as_str()).into());
                return;
            }
            let base = load_config_base(config_path.as_str());  
            //combox.set_selected_index(base.font.main_index as u32);
            let _ = update_ui(&ui,&base);
            println!("加载的配置文件完成: {}", config_path);
            let _ = ui.set_outtext(format!("加载的配置文件完成: {}", config_path).into());
        }
    }) ;

    let ui_weak_save: slint::Weak<MainWindow> = ui_weak.clone();
    // 保存配置
    ui.on_save_config(move || {
        // 在闭包中使用弱引用访问UI
        if let Some(ui) = ui_weak_save.upgrade() { 
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
            let base = update_base(&ui);
            if let Err(e) = save_config_base(&base,config_path.as_str()) {
                println!("保存配置文件失败: {:?}", e);
                let _ = ui.set_outtext(format!("保存配置文件失败: {:?}", e).into());
                return;
            }
            println!("配置文件已保存: {}", config_path);
            let _ = ui.set_outtext(format!("配置文件已保存: {}", config_path).into());
        }
    }) ;
    ui.run().unwrap();
}
fn load_config_base(path: &str) -> Base {
    let base = config::Base::from_json_file (path);
    base
}

fn save_config_base(base: &Base,path: &str) -> Result<(), Box<dyn std::error::Error>> {
    config::Base::save_json(base,path)?;
    Ok(())
}

fn update_config(ui: &MainWindow) -> Parameter  {
    let mut base = load_config_base("./config.json");
    base.bg_color = ui.get_background_color().to_string();
    base.draw_color = ui.get_font_color().to_string();
    base.line_color = ui.get_draw_color().to_string(); 
    base.bookname = ui.get_book_name().to_string();
    base.bookinputpath = ui.get_input_path().to_string();
    base.bookoutputpath = ui.get_output_path().to_string();
    base.main_font_path = ui.get_font_path().to_string();
    base.backup_font_path = ui.get_font_backup_path().to_string();
    base.compressratio = ui.get_compress_ratio() as u8;
    base.page_width_mm = ui.get_page_width_mm() as f32;
    base.page_height_mm = ui.get_page_height_mm() as f32;
    base.column_count = ui.get_column_count() as usize;
    let center_width_mm = ui.get_center_width_mm() as f32;
    let page_top_margin_mm = ui.get_page_top_margin_mm() as f32;
    let left_margin_mm = ui.get_page_left_margin_mm() as f32;
    let tail_margin_mm = ui.get_tail_margin_mm() as f32;   
    let col_width_mm = (base.page_width_mm - center_width_mm - left_margin_mm * 2.0) 
                           / base.column_count as f32;
    base.center_width_mm = center_width_mm;
    base.page_top_margin_mm = page_top_margin_mm;
    base.page_left_margin_mm = left_margin_mm;
    base.page_bottom_margin_mm = left_margin_mm;
    base.page_right_margin_mm = left_margin_mm;
    base.tail_margin_mm = tail_margin_mm;    
    base.tail_long_offset_mm = tail_margin_mm * 0.4;
    base.tail_short_offset_mm = tail_margin_mm * 0.25;
    base.title_font_size_pt = (center_width_mm * 0.45 * MM_TO_PT).round() as f32;
    base.content_font_size_pt = (col_width_mm * 0.6 * MM_TO_PT).round() as f32; 
    let param: Parameter = config::default(base);
    param
}

fn update_base(ui: &MainWindow) -> Base{
    let mut base = config::Base::default();
    base.bg_color = ui.get_background_color().to_string();
    base.draw_color = ui.get_font_color().to_string();
    base.line_color = ui.get_draw_color().to_string(); 
    base.bookname = ui.get_book_name().to_string();
    base.bookinputpath = ui.get_input_path().to_string();
    base.bookoutputpath = ui.get_output_path().to_string();
    base.main_font_path = ui.get_font_path().to_string();
    base.backup_font_path = ui.get_font_backup_path().to_string();
    base.compressratio = ui.get_compress_ratio() as u8;
    base.page_width_mm = ui.get_page_width_mm() as f32;
    base.page_height_mm = ui.get_page_height_mm() as f32;
    base.column_count = ui.get_column_count() as usize;
    let center_width_mm = ui.get_center_width_mm() as f32;
    let page_top_margin_mm = ui.get_page_top_margin_mm() as f32;
    let left_margin_mm = ui.get_page_left_margin_mm() as f32;
    let tail_margin_mm = ui.get_tail_margin_mm() as f32;   
    let col_width_mm = (base.page_width_mm - center_width_mm - left_margin_mm * 2.0) 
                           / base.column_count as f32;
    base.center_width_mm = center_width_mm;
    base.page_top_margin_mm = page_top_margin_mm;
    base.page_left_margin_mm = left_margin_mm;
    base.page_bottom_margin_mm = left_margin_mm;
    base.page_right_margin_mm = left_margin_mm;
    base.tail_margin_mm = tail_margin_mm;    
    base.tail_long_offset_mm = tail_margin_mm * 0.4;
    base.tail_short_offset_mm = tail_margin_mm * 0.25;
    base.title_font_size_pt = (center_width_mm * 0.45 * MM_TO_PT).round() as f32;
    base.content_font_size_pt = (col_width_mm * 0.6 * MM_TO_PT).round() as f32;    
    base
}
fn update_ui(ui: &MainWindow,base: &Base) {
    ui.set_page_width_mm(base.page_width_mm.into());
    ui.set_page_height_mm(base.page_height_mm.into());
    ui.set_center_width_mm((base.center_width_mm as i32).into());
    ui.set_page_top_margin_mm((base.page_top_margin_mm as i32).into());
    ui.set_page_left_margin_mm((base.page_left_margin_mm as i32).into());
    ui.set_tail_margin_mm((base.tail_margin_mm as i32).into());
    ui.set_column_count((base.column_count as i32).into());
    ui.set_font_color(base.draw_color.clone().into());
    ui.set_background_color(base.bg_color.clone().into());
    ui.set_draw_color(base.line_color.clone().into());
    ui.set_font_path(base.main_font_path.to_string().into());
    ui.set_font_backup_path(base.backup_font_path.to_string().into());
    ui.set_input_path(base.bookinputpath.to_string().into());
    ui.set_output_path(base.bookoutputpath.to_string().into());
    ui.set_book_name(base.bookname.to_string().into());
    ui.set_compress_ratio(base.compressratio.into());
}
