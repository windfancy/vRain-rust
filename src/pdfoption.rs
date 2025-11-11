use printpdf::*;
use crate::utils;
use crate::config::*;
use std::path::Path;
use std::sync::Arc;
use rayon::prelude::*;
pub fn create_pdf(page: &Pager,drawcolor:&DrawColor,fileinfo:&FileInfo) {    
    
    // 设置主要参数
    let page_width_mm = page.page_width_mm;
    let page_height_mm = page.page_height_mm;
    
    let column_count = page.column_count  as usize;    
    let book_name = fileinfo.name.as_str();
    let book_creater = fileinfo.creater.as_str();
    let main_font_path = fileinfo.main_path.as_str();
    let backup_font_path = fileinfo.backup_path.as_str();
    let input_path = fileinfo.inputpath.as_str();
    let output_path = fileinfo.outputpath.as_str();
    
    let content = get_content_loc(&page);
    let pagination = get_pagination_loc(&page);
    let title = get_title_loc(&page);

    let count_per_column = content.max_chars  as usize;
    let fontcolor = color_to_rgb(&drawcolor.draw.as_str());
    let bgcolor = color_to_rgb(&drawcolor.bg.as_str());
    let linecolor = color_to_rgb(&drawcolor.line.as_str());
    //  direction: 0 横向 1 纵向
    let direction: u8 = if page_width_mm > page_height_mm {0} else {1};
    let canvas = if direction == 0 { get_canvas_horizontal(&page) } else { get_canvas_vertical(&page) };
    let tail = if direction == 0 { get_tail_horizontal(&page) } else { get_tail_vertical(&page)};
    // 检查输入文件是否存在
    if !Path::new(&input_path).exists() {
        println!("错误：输入文件不存在: {}", input_path);
        return;
    }    
    // 获取文本内容，处理可能的错误
    let text = match utils::get_txt(&input_path) {
        Ok(content) => content,
        Err(e) => {
            println!("读取输入文件失败: {:?}", e);
            return;
        }
    };
    // 生成[章][页][行]数组
    let txt_pages = utils::process_text(&text, count_per_column, column_count);
    //pdf文档参数初始化
    let mut doc = PdfDocument::new(book_name);
    
    //设置背景层
    let background_layer = Layer {
        name: "Background".to_string(),
        creator: book_creater.to_string(),
        intent: LayerIntent::View,
        usage: LayerSubtype::Artwork,
    };
    let background_layer_id = doc.add_layer(&background_layer);
    //设置文本层
    let text_layer = Layer {
        name: "Text Content".to_string(),
        creator: book_creater.to_string(),
        intent: LayerIntent::Design,
        usage: LayerSubtype::Artwork,
    };
    let text_layer_id = doc.add_layer(&text_layer);
    // 增加主字体
    let main_font_bytes = std::fs::read(&main_font_path).unwrap();
    let font_byte_slice_main: &[u8] = &main_font_bytes;
    let main_font =
        printpdf::ParsedFont::from_bytes(font_byte_slice_main, 0, 
                               &mut Vec::new()).unwrap();
    let main_font_id: FontId = doc.add_font(&main_font);
    // 增加备用字体
    let backup_font_bytes = std::fs::read(&backup_font_path).unwrap();
    let font_byte_slice_backup: &[u8] = &backup_font_bytes;
    let backup_font =
        printpdf::ParsedFont::from_bytes(font_byte_slice_backup, 1, 
                               &mut Vec::new()).unwrap();
    let backup_font_id = doc.add_font(&backup_font);
    let cover_font_id = main_font_id.clone();

    //let mut pages = vec![];
    // 将不可变参数包装成Arc，以便在多线程间安全共享
    let canvas_arc = Arc::new(canvas.clone());
    let tail_arc = Arc::new(tail);
    let pagination_arc = Arc::new(pagination.clone());
    let content_arc = Arc::new(content.clone());
    let title_arc = Arc::new(title);

    let main_font_arc = Arc::new(main_font_id);
    let backup_font_arc = Arc::new(backup_font_id);
    let bg_layer_arc = Arc::new(background_layer_id);
    let txt_layer_arc = Arc::new(text_layer_id);

    let mut pages: Vec<(usize,usize, PdfPage)> = txt_pages
        .par_iter()  // 并行迭代器（替换原for循环）
        .enumerate()
        .flat_map(|(chap_idx, pages)| {
            let total_pages_before = txt_pages[0..chap_idx]
                .iter()
                .map(|chap| chap.len())  // 每个章的页数
                .sum::<usize>();
            let canvas_arc_clone = Arc::clone(&canvas_arc);
            let tail_arc_clone = Arc::clone(&tail_arc);
            let pagination_arc_clone = Arc::clone(&pagination_arc);
            let content_arc_clone = Arc::clone(&content_arc);
            let title_arc_clone = Arc::clone(&title_arc);
            let main_font_arc_clone = Arc::clone(&main_font_arc);
            let backup_font_arc_clone = Arc::clone(&backup_font_arc);
            let bg_layer_arc_clone = Arc::clone(&bg_layer_arc);
            let txt_layer_arc_clone = Arc::clone(&txt_layer_arc);

            let bgcolor_outer = bgcolor.clone();  // 关键：外层先克隆
            let linecolor_outer = linecolor.clone();
            let fontcolor_outer = fontcolor.clone();
            let direction_outer = direction.clone();
            let book_name_outer = book_name.clone();
            pages.par_iter()
                 .enumerate() // 页索引 (page_idx)
                 .map(move |(page_idx, lines)| {
                let canvas_clone = Arc::clone(&canvas_arc_clone);  // 为当前线程克隆
                let tail_clone = Arc::clone(&tail_arc_clone);
                let pagination_clone = Arc::clone(&pagination_arc_clone);
                let content_clone = Arc::clone(&content_arc_clone);
                let title_clone = Arc::clone(&title_arc_clone);
                let main_font_clone = Arc::clone(&main_font_arc_clone);
                let backup_font_clone = Arc::clone(&backup_font_arc_clone);
                let bg_layer_clone = Arc::clone(&bg_layer_arc_clone);
                let txt_layer_clone = Arc::clone(&txt_layer_arc_clone);

                // 非Arc变量的克隆保持不变
                let linecolor_clone = linecolor_outer.clone();
                let fontcolor_clone = fontcolor_outer.clone();
                let bgcolor_clone = bgcolor_outer.clone();
                let direction_clone = direction_outer.clone();
                let book_name_clone = book_name_outer.to_string();
                
                let mut ops: Vec<Op> = Vec::new();            
                // 制作模板（背景层）            
                ops.append(&mut add_template(
                            page_width_mm,
                            page_height_mm,
                            &canvas_clone,
                            &tail_clone,
                            bgcolor_clone,
                            linecolor_clone,
                            column_count,
                            direction_clone,
                            bg_layer_clone.as_ref().clone()));
                // 处理文本内容（文本层）
                ops.push(Op::BeginLayer {layer_id: txt_layer_clone.as_ref().clone(),});
                // 添加页码
                let total_page_idx = total_pages_before + page_idx + 1;
                let page_num_ops = add_pagenumber_text(                
                    &format!("{}", total_page_idx), 
                    &pagination_clone,
                    &fontcolor_clone,
                    &backup_font_clone
                );
                ops.extend(page_num_ops);          
                // 添加标题            
                let title_ops = add_title_text(
                    &title_clone,   
                    &book_name_clone, 
                    &fontcolor_clone,               
                    &main_font_clone,
                    &backup_font_clone
                );
                ops.extend(title_ops);
                
                // 添加内容
                let content_ops = add_centent_text(
                    &lines, 
                    &content_clone,
                    &fontcolor_clone,
                    &main_font_clone,
                    &backup_font_clone
                );
                ops.extend(content_ops);
                
                ops.push(Op::EndLayer {
                    layer_id: txt_layer_clone.as_ref().clone(),
                });
                // 创建页面并返回（包含页码用于排序）
                    (
                        chap_idx,
                        page_idx,
                        PdfPage::new(
                            Mm(page_width_mm),
                            Mm(page_height_mm),
                            ops
                        )
                    )
                })
            })
            .collect();        
         
    // 按页码排序页面（确保顺序正确）
    sort_processed_pages(&mut pages);
    // 排序后直接提取 PdfPage（顺序已保证）
    let mut ordered_pages: Vec<PdfPage> = pages.into_iter().map(|(_, _, page)| page).collect();
    //  direction: 0 横向 1 纵向
    if direction == 1 {
        let image_bytes = include_bytes!("./cover.png");
        let image = RawImage::decode_from_bytes(image_bytes, &mut Vec::new()).unwrap();        
        let image_id = doc.add_image(&image);
        let cover_txt = book_name;
        let first_page = add_cover(image_id, 
                        cover_font_id,
                                    image,
                                    cover_txt,
                                    page_width_mm, 
                                    page_height_mm);
        ordered_pages.insert(0, first_page);
        
    }    
    // 将所有页面添加到文档
    doc.with_pages(ordered_pages);
    println!("正在保存 {}", output_path);
    // 保存PDF文件
    let bytes = doc.save(&PdfSaveOptions::default(), &mut Vec::new());
    
    std::fs::write(output_path, bytes)
        .expect("Failed to write PDF file");    
    println!("Created {}", output_path);
}

// 按 (章索引, 页索引) 升序排序
fn sort_processed_pages(processed_pages: &mut Vec<(usize, usize, PdfPage)>) {    
    processed_pages.sort_by_key(|&(chap_idx, page_idx, _)| (chap_idx, page_idx));
}

fn add_cover(image_id: XObjectId,
    main_font_id:FontId,
    image: RawImage,
    txt: &str,
    page_width_mm: f32, 
    page_height_mm: f32) ->PdfPage{
    //设置封面图像
    
    let mut ops1 = Vec::new();
    let dpi: f32 = 72.0; // 假设图片DPI为72（1英寸=25.4毫米）
    let image_width_mm: f32 = image.width as f32 * 25.4 / dpi;
    let image_height_mm: f32 = image.height as f32 * 25.4 / dpi;

    let scale_x = page_width_mm / image_width_mm;
    let scale_y = page_height_mm / image_height_mm;
    //let scale = scale_x.min(scale_y); // 取最小缩放比例，避免拉伸

    ops1.push(Op::UseXobject {
        id: image_id.clone(),
        transform: XObjectTransform {
            translate_x: Some(Pt(0.0)),
            translate_y: Some(Pt(0.0)),
            rotate: Some(XObjectRotation {
                angle_ccw_degrees: 0.0,
                rotation_center_x: Px(0),
                rotation_center_y: Px(0),
            }),
            scale_x: Some(scale_x),
            scale_y: Some(scale_y),
            dpi: Some(dpi),
        },
    });
    //let txt = "庄子";
    let font_size: f32 = page_width_mm / 10.0 * MM_TO_PT;
    let char_x: Pt = Pt(page_width_mm / 5.0 * MM_TO_PT - font_size / 2.0);
    let offset_y = Pt(font_size);
    let mut char_y: Pt = Pt(page_height_mm * 7.0 / 8.0 * MM_TO_PT - font_size / 2.0);
    for char in txt.chars() {
        let ops = add_vertical_text(
            &main_font_id,
            font_size,
            char_x,
            char_y,
            &char.to_string(),
            0.0,
        );
        ops1.extend(ops);
        char_y -= offset_y;
    }
    PdfPage::new(
     Mm(page_width_mm),
     Mm(page_height_mm),
     ops1
    )
}
pub fn create_pdf_template(page: &Pager,drawcolor:&DrawColor,fileinfo:&FileInfo) {

    let page_width_mm = page.page_width_mm;
    let page_height_mm = page.page_height_mm;
    let column_count = page.column_count  as usize; 
    let bgcolor = color_to_rgb(drawcolor.bg.as_str());   
    let linecolor = color_to_rgb(drawcolor.line.as_str());
    let book_name = fileinfo.name.as_str();
    let direction: u8 = if page_width_mm > page_height_mm {0} else {1};

    let canvas = if direction == 0 { get_canvas_horizontal(&page) } else { get_canvas_vertical(&page) };

    let tail = if direction == 0 { get_tail_horizontal(&page) } else { get_tail_vertical(&page)}; 
    let mut doc = PdfDocument::new(book_name);
    //设置背景层
    let background_layer = Layer {
        name: "Background".to_string(),
        creator: "template".to_string(),
        intent: LayerIntent::View,
        usage: LayerSubtype::Artwork,
    };
    let background_layer_id = doc.add_layer(&background_layer);
    let mut ops: Vec<Op> = Vec::new();            
            // 制作模板（背景层）            
    ops.append(&mut add_template(page_width_mm,
                page_height_mm,
                &canvas,
                &tail,
                bgcolor,
                linecolor,
                column_count,
                direction,
                background_layer_id.clone()));
    let ordered_pages = PdfPage::new(
        Mm(page_width_mm),
        Mm(page_height_mm),
        ops
    );
    doc.with_pages(vec![ordered_pages]);
    // 保存PDF文件
    let bytes = doc.save(&PdfSaveOptions::default(), &mut Vec::new());
    
    std::fs::write("./template.pdf", bytes)
        .expect("Failed to write PDF file");    
    println!("Created template.pdf");

}

// 添加内容文本
fn add_vertical_text(
        fontid: &FontId,
        fontsize: f32,
        char_x: Pt,
        char_y: Pt,
        char : &str,
        char_rotate: f32,)->Vec<Op>{

    let mut ops = vec![];
    ops.push(Op::SetFontSize { font: fontid.clone(), size: Pt(fontsize) });    
    ops.push(Op::SetTextMatrix {matrix: TextMatrix::TranslateRotate(char_x, char_y,char_rotate) });
    ops.push(Op::WriteText {items: vec![TextItem::Text(char.to_string())],font: fontid.clone()});   
    ops
}
fn add_pagenumber_text(
        text: &str,
        pagination: &Pagination,
        fontcolor:&Color,
        font_id: &FontId)->Vec<Op>{

    let mut ops = vec![];

    let fontsize =pagination.font_size_pt;
    let color_rbg = fontcolor.clone();
    ops.push(Op::SetFillColor { col: color_rbg }); 

    let loc_start_x_pt = pagination.loc_start_x_pt;
    let loc_start_y_pt = pagination.loc_start_y_pt;
    let space_y_pt = Pt(pagination.font_size_pt * FONT_OFFSET_SCALE);
    let mut char_x;
    let mut char_y;
    //let char_x = Pt(400.0);
    //let char_y = Pt(447.0);
    for (i, char) in text.chars().enumerate(){
        let char_content: char = utils::replace_char(char);
        char_x = loc_start_x_pt;
        char_y = loc_start_y_pt - space_y_pt * i as f32;
        ops.append(&mut add_vertical_text(font_id, 
                            fontsize, 
                            char_x, 
                            char_y, 
                            &char_content.to_string(), 
                            0.0));
    }
    //println!("ops: {:?}", ops);
    ops
}
fn add_title_text(
        t: &Title,
        txt: &str, 
        fontcolor:&Color,       
        font_id: &FontId,
        font_backup_id: &FontId)->Vec<Op>{
        
    let mut ops = vec![];

    let fontsize =t.font_size_pt;
    let color_rbg = fontcolor.clone();
    ops.push(Op::SetFillColor { col: color_rbg }); 
    

    //let txt = "庄子";
    let mut char_x;
    let mut char_y;
    for (i, char) in txt.chars().enumerate(){
        if (i + 1) as i32 >= t.max_chars {
            break;
        }
        char_x =t.loc_start_x_pt;
        char_y = t.loc_start_y_pt + t.space_y_pt * i as f32;
        let char_content: char = utils::replace_char(char);
        match utils::is_punctuation(char_content) {
            0 => {// 是无读字符 
                    ops.append(&mut add_vertical_text(font_backup_id, 
                        fontsize, 
                        char_x, char_y, 
                        &char_content.to_string(), 
                        0.0));
                }
            _ => {
                    ops.append(&mut add_vertical_text(font_id, 
                        fontsize, 
                        char_x, char_y, 
                        &char_content.to_string(), 
                        0.0));                        
                } 
        }
    }
    ops
}

// 添加内容文本
fn add_centent_text(
        texts: &[String],
        content: &Content,
        fontcolor: &Color,
        font_id: &FontId,
        font_backup_id: &FontId,)->Vec<Op>
    {
    let mut ops = vec![];
    let fontsize =content.font_size_pt;
    let color_rbg = fontcolor.clone();
    ops.push(Op::SetFillColor { col: color_rbg }); 

    let mut char_x ;
    let mut char_y ;
    let mut loc_x_pt;
    let mut loc_y_pt;

    for (col, linetxt) in texts.iter().enumerate(){  
        let mut count = 0; 
        loc_x_pt = content.loc_x_pt[col];            
        loc_y_pt = content.loc_y_pt;

        for (_, char) in linetxt.chars().enumerate(){
            if count >= content.max_chars {
                //count = 0;
                break;
            }
            let char_content: char = utils::replace_char(char);
            match utils::is_punctuation(char_content) {
                0 => {// 无读字符
                    char_x = loc_x_pt;
                    char_y = loc_y_pt + content.space_y_pt * count as f32;                    
                    ops.append(&mut add_vertical_text(font_backup_id, 
                                    fontsize, 
                                    char_x, char_y,  
                                    &char.to_string(), 
                                    0.0));
                    count += 1;               
                }
                1 => {// 标点字符
                    char_x = loc_x_pt + Pt(fontsize);
                    char_y = loc_y_pt + content.space_y_pt * (count - 1) as f32;                    
                    ops.append(&mut add_vertical_text(font_backup_id, 
                                    content.pun_font_size_pt, 
                                    char_x, char_y, 
                                    &char.to_string(), 
                                    0.0));
                }
                3 =>{// 旋转字符
                    char_x = loc_x_pt;
                    char_y = loc_y_pt + content.space_y_pt * count as f32 + Pt(fontsize * PUN_PUB);                    
                    ops.append(&mut add_vertical_text(font_backup_id, 
                                    fontsize, 
                                    char_x, char_y, 
                                    &char.to_string(), 
                                    -90.0));
                    count += 1;   

                }
                _ => {// 正常字符
                    char_x = loc_x_pt;
                    char_y = loc_y_pt + content.space_y_pt * count as f32;                    
                    ops.append(&mut add_vertical_text(font_id, 
                                    fontsize, 
                                    char_x, char_y, 
                                    &char.to_string(), 
                                    0.0));
                    count += 1;               
                }            
            }
            //print!("char: {}, char_x: {:?}, char_y:{:?}\n",char, char_x, char_y);
        }
    }
    ops
}


fn add_template(page_width_mm: f32, 
                    page_height_mm: f32,
                    canvas:&Canvas,
                    tail:& Tail,
                    bg_color:Color,
                    line_color:Color,
                    column_count: usize,
                    direction: u8,
                    bg_layer_id: LayerInternalId)->Vec<Op>{
    
    let mut ops = vec![];

    //print!("绘制底色\n");
    //绘制底色
    ops.push(Op::BeginLayer {layer_id: bg_layer_id.clone()});
    ops.push(Op::SetFillColor { col: bg_color });
    ops.push(Op::DrawPolygon {
        polygon: printpdf::Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    LinePoint {
                        p: Point {
                            x: Pt(0.0),
                            y: Pt(0.0), // Top left
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Mm(page_width_mm).into_pt(),
                            y: Pt(0.0), // Top right
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Mm(page_width_mm).into_pt(),
                            y: Mm(page_height_mm).into_pt(), // Bottom right
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(0.0),
                            y: Mm(page_height_mm).into_pt(), // Bottom left
                        },
                        bezier: false,
                    },
                ],
            }],
            mode: printpdf::PaintMode::Fill,
            winding_order: printpdf::WindingOrder::NonZero,
        },
    });


    //print!("绘制页面外边框\n");
    // 绘制页面外边框    
    ops.push(Op::SetOutlineColor { col: line_color.clone() });
    ops.push(Op::SetOutlineThickness { pt: canvas.line_width_pt });
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: Point {
                        x: canvas.point_left_bottom.x - canvas.line_offset_pt,
                        y: canvas.point_left_bottom.y - canvas.line_offset_pt,
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: canvas.point_left_top.x - canvas.line_offset_pt,
                        y: canvas.point_left_top.y + canvas.line_offset_pt,
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: canvas.point_right_top.x + canvas.line_offset_pt,
                        y: canvas.point_right_top.y + canvas.line_offset_pt,
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: canvas.point_right_bottom.x + canvas.line_offset_pt,
                        y: canvas.point_right_bottom.y - canvas.line_offset_pt,
                    },
                    bezier: false,
                },
            ],
            is_closed: true,
        } 
    } ); 
    
    //绘制鱼尾中线
    if direction == 0 {  
        //print!("绘制鱼尾中线");  
        ops.push(Op::DrawLine { 
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point {
                            x: Mm(page_width_mm / 2.0).into_pt(),
                            y: canvas.point_center_left_bottom.y, // Bottom left
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Mm(page_width_mm / 2.0).into_pt(),
                            y: tail.point_line_down_left.y, 
                        },
                        bezier: false,
                    },
                ],
                is_closed: false,
            } } );

        ops.push(Op::DrawLine { 
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point {
                            x: Mm(page_width_mm / 2.0).into_pt(),
                            y: tail.point_line_up_left.y, 
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Mm(page_width_mm / 2.0).into_pt(),
                            y: canvas.point_center_left_top.y, 
                        },  
                        bezier: false,
                    },
                ],
                is_closed: false,
            } } );
    }
    //print!("绘制页面内边框");
    // 绘制页面内边框
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: canvas.point_left_bottom,
                    bezier: false,
                },
                LinePoint {
                    p: canvas.point_left_top,
                    bezier: false,
                },
                LinePoint {
                    p: canvas.point_right_top,
                    bezier: false,
                },
                LinePoint {
                    p: canvas.point_right_bottom,
                    bezier: false,
                },
            ],
            is_closed: true,
        } } ); 
    
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: canvas.point_center_left_bottom,
                    bezier: false,
                },
                LinePoint {
                    p: canvas.point_center_left_top,
                    bezier: false,
                },
                LinePoint {
                    p: canvas.point_center_right_top,
                    bezier: false,
                },
                LinePoint {
                    p: canvas.point_center_right_bottom,
                    bezier: false,
                },
            ],
            is_closed: true,
        } } ); 
    

    //print!("绘制列线\n");
    for i in 0..column_count{
        if i < column_count /2 || direction ==1 {
            ops.push(Op::DrawLine { 
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point {
                                x: canvas.point_left_bottom.x 
                                   + canvas.column_width_pt * i as f32,
                                y: canvas.point_left_bottom.y, // Bottom left
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: canvas.point_left_top.x + canvas.column_width_pt * i as f32,
                                y: canvas.point_left_top.y, // Top left
                            },
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                } } ); 
        }else{
            ops.push(Op::DrawLine { 
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point {
                                x: canvas.point_right_bottom.x 
                                   - canvas.column_width_pt 
                                   * (i - column_count / 2) as f32,
                                y: canvas.point_right_bottom.y, // Bottom left
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: canvas.point_right_top.x 
                                   - canvas.column_width_pt 
                                   * (i - column_count / 2) as f32,
                                y: canvas.point_right_top.y, // Top left
                            },
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                } } ); 
        }
    }
    //绘制鱼尾上下细线
    // 下鱼尾
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: tail.point_line_down_left,
                    bezier: false,
                },
                LinePoint {
                    p: tail.point_line_down_right,
                    bezier: false,
                },
            ],
            is_closed: false,
        } } ); 
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: tail.point_line_up_left,
                    bezier: false,
                },
                LinePoint {
                    p: tail.point_line_up_right,
                    bezier: false,
                },
            ],
            is_closed: false,
        } } );

    //绘制鱼尾
    // 上鱼尾
    ops.push(Op::SetFillColor { col: line_color.clone()});
    ops.push(Op::DrawPolygon {
        polygon: printpdf::Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    LinePoint {
                        p: tail.point_up_left_top,
                        bezier: false,
                    },
                    LinePoint {
                        p: tail.point_up_right_top,  
                        bezier: false,
                    },
                    LinePoint {
                        p: tail.point_up_right_bottom,
                        bezier: false,
                    },
                    LinePoint {
                        p: tail.point_up_center,
                        bezier: false,
                    },
                    LinePoint {
                        p: tail.point_up_left_bottom,
                        bezier: false,
                    },
                ],
            }],
            mode: printpdf::PaintMode::Fill,
            winding_order: printpdf::WindingOrder::NonZero,
        },
    }); 
    // 下鱼尾
    ops.push(Op::DrawPolygon {
        polygon: printpdf::Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    LinePoint {
                        p: tail.point_down_left_bottom,
                        bezier: false,
                    },
                    LinePoint {
                        p: tail.point_down_right_bottom,  
                        bezier: false,
                    },
                    LinePoint {
                        p: tail.point_down_right_top,
                        bezier: false,
                    },
                    LinePoint {
                        p: tail.point_down_center,
                        bezier: false,
                    },
                    LinePoint {
                        p: tail.point_down_left_top,
                        bezier: false,
                    },
                ],
            }],
            mode: printpdf::PaintMode::Fill,
            winding_order: printpdf::WindingOrder::NonZero,
        },
    });   
    ops.push(Op::EndLayer {layer_id: bg_layer_id.clone()});  
    ops
}


