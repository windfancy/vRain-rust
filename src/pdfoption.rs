use printpdf::*;
use crate::utils;
use crate::config::*;
use std::path::Path;
use std::sync::Arc;
use std::thread;

pub fn create_pdf(param: &Parameter) {    
    
    // 设置主要参数
    let page_width_mm = param.pageinfo.page_width_mm;
    let page_height_mm = param.pageinfo.page_height_mm;
    let count_per_column = param.content.max_chars  as usize;
    let column_count = param.pageinfo.column_count  as usize;    
    let book_name = param.book.name.as_str();
    let book_creater = param.book.creater.as_str();
    let main_font_path = param.font.main_path.as_str();
    let backup_font_path = param.font.backup_path.as_str();
    let input_path = param.file.inputpath.as_str();
    let output_path = param.file.outputpath.as_str();
    let template =Template{
        page_width_mm: param.pageinfo.page_width_mm,
        page_height_mm: param.pageinfo.page_height_mm,
        bgcolor: param.color.bg.clone(),
        linecolor: param.color.line.clone(),
        tail: param.tail.clone(),
        canvas: param.canvas.clone(),
    };
    let pagination = param.pagination.clone();
    let title: Title = param.title.clone();
    let content = param.content.clone();
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
    // 按每count_per_column个字符一组分割
    let lines = utils::split_into_lines(&text, count_per_column);
    println!("总共有{}组", lines.len());
    // 2. 再按每column_count行分割成页
    let txt_pages = utils::split_into_pages(&lines, column_count);
    println!("共生成 {} 页", txt_pages.len());
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
    let main_font_id = doc.add_font(&main_font);
    // 增加备用字体
    let backup_font_bytes = std::fs::read(&backup_font_path).unwrap();
    let font_byte_slice_backup: &[u8] = &backup_font_bytes;
    let backup_font =
        printpdf::ParsedFont::from_bytes(font_byte_slice_backup, 1, 
                               &mut Vec::new()).unwrap();
    let backup_font_id = doc.add_font(&backup_font);

    //let mut pages = vec![];
    // 将不可变参数包装成Arc，以便在多线程间安全共享
    let template_arc = Arc::new(template.clone());
    let pagination_arc = Arc::new(pagination.clone());
    let content_arc = Arc::new(content.clone());
    let title_arc = Arc::new(title);
    let main_font_arc = Arc::new(main_font_id);
    let backup_font_arc = Arc::new(backup_font_id);
    let bg_layer_arc = Arc::new(background_layer_id);
    let txt_layer_arc = Arc::new(text_layer_id);

    // 用于存储线程句柄
    let mut handles = vec![];

    for (page_num, page_txt) in txt_pages.iter().enumerate() {

        let template_clone = Arc::clone(&template_arc);
        let pagination_clone = Arc::clone(&pagination_arc);
        let content_clone = Arc::clone(&content_arc);
        let title_clone = Arc::clone(&title_arc);
        let page_txt = page_txt.clone();
        let main_font_clone = Arc::clone(&main_font_arc);
        let backup_font_clone = Arc::clone(&backup_font_arc);
        let bg_layer_clone = Arc::clone(&bg_layer_arc);
        let txt_layer_clone = Arc::clone(&txt_layer_arc);
        let book_name_clone: String = book_name.to_string();

        // 创建线程
        let handle = thread::spawn(move || {
            //println!("线程 {:?} 正在制作第{}页", thread::current().id(), page_num + 1);            
            let mut ops: Vec<Op> = Vec::new();            
            // 制作模板（背景层）            
            ops.append(&mut add_template(&template_clone,
                       column_count,
                       bg_layer_clone.as_ref().clone()));
            // 处理文本内容（文本层）
            ops.push(Op::BeginLayer {layer_id: txt_layer_clone.as_ref().clone(),});
            // 添加页码
            let page_num_ops = add_pagenumber_text(                
                &format!("{}", page_num + 1), 
                &pagination_clone,
                &backup_font_clone
            );
            ops.extend(page_num_ops);          
            // 添加标题            
            let title_ops = add_title_text(
                &title_clone,   
                &book_name_clone,                
                &main_font_clone,
                &backup_font_clone
            );
            ops.extend(title_ops);
            
            // 添加内容
            let content_ops = add_centent_text(
                &page_txt, 
                &content_clone,
                column_count,
                &main_font_clone,
                &backup_font_clone
            );
            ops.extend(content_ops);
            
            ops.push(Op::EndLayer {
                layer_id: txt_layer_clone.as_ref().clone(),
            });
            // 创建页面并返回（包含页码用于排序）
            (
                page_num,
                PdfPage::new(
                    Mm(page_width_mm),
                    Mm(page_height_mm),
                    ops
                )
            )
        });        
        handles.push(handle);
    }
    // 收集所有线程的结果
    let mut pages = vec![];
    for handle in handles {
        // 等待线程完成并获取结果
        let (page_num, page) = handle.join().expect("线程执行出错");
        pages.push((page_num, page));
    }
    
    // 按页码排序页面（确保顺序正确）
    pages.sort_by_key(|&(num, _)| num);
    let ordered_pages: Vec<PdfPage> = pages.into_iter().map(|(_, page)| page).collect();
    
    // 将所有页面添加到文档
    doc.with_pages(ordered_pages);
    println!("正在保存 {}", output_path);
    // 保存PDF文件
    let bytes = doc.save(&PdfSaveOptions::default(), &mut Vec::new());
    
    std::fs::write(output_path, bytes)
        .expect("Failed to write PDF file");    
    println!("Created {}", output_path);
}

// 添加内容文本
fn add_text(
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
        font_id: &FontId)->Vec<Op>{

    let mut ops = vec![];

    let fontsize =pagination.font_size_pt;
    let fontcolor = &pagination.font_color;
    let color_rbg = color_to_rgb(&fontcolor);
    ops.push(Op::SetFillColor { col: color_rbg }); 

    let char_x = Pt(pagination.loc_start_x_pt.0 - text.len() as f32 * fontsize / 2.0);
    let char_y = pagination.loc_start_y_pt;
    let pagenumber_text = utils::replace_numbers_with_chinese(text);
    //let char_x = Pt(400.0);
    //let char_y = Pt(447.0);
    ops.append(&mut add_text(font_id, 
                        fontsize, 
                        char_x, 
                        char_y, 
                        &pagenumber_text, 
                        0.0));
    //println!("ops: {:?}", ops);
    ops
}
fn add_title_text(
        t: &Title,
        txt: &str,        
        font_id: &FontId,
        font_backup_id: &FontId)->Vec<Op>{
        
    let mut ops = vec![];

    let fontsize =t.font_size_pt;
    let fontcolor = &t.font_color;
    let color_rbg = color_to_rgb(&fontcolor);
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
                    ops.append(&mut add_text(font_backup_id, 
                        fontsize, 
                        char_x, char_y, 
                        &char_content.to_string(), 
                        0.0));
                }
            _ => {
                    ops.append(&mut add_text(font_id, 
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
        column_count: usize,
        font_id: &FontId,
        font_backup_id: &FontId,)->Vec<Op>
    {
    let mut ops = vec![];
    let fontsize =content.font_size_pt;
    let fontcolor = &content.font_color;
    let color_rbg = color_to_rgb(&fontcolor);
    ops.push(Op::SetFillColor { col: color_rbg }); 

    let mut char_x ;
    let mut char_y ;
    let mut loc_x_pt;
    let mut loc_y_pt;
    let mut offset_x;

    for (col, linetxt) in texts.iter().enumerate(){  
        let mut count = 0; 
        if col < (column_count as usize /2) {
            loc_x_pt = content.loc_start_x1_pt;            
            loc_y_pt = content.loc_start_y1_pt;
            offset_x = content.space_x_pt * col as f32;
        }else {
            loc_x_pt = content.loc_start_x2_pt;
            loc_y_pt = content.loc_start_y2_pt;
            offset_x = content.space_x_pt * (col - column_count as usize /2) as f32;
        }
        for (_, char) in linetxt.chars().enumerate(){
            if count >= content.max_chars {
                count = 0;
                break;
            }
            let char_content: char = utils::replace_char(char);
            match utils::is_punctuation(char_content) {
                0 => {// 无读字符
                    char_x = loc_x_pt + offset_x;
                    char_y = loc_y_pt + content.space_y_pt * count as f32;                    
                    ops.append(&mut add_text(font_backup_id, 
                                    fontsize, 
                                    char_x, char_y,  
                                    &char.to_string(), 
                                    0.0));
                    count += 1;               
                }
                1 => {// 标点字符
                    char_x = loc_x_pt + offset_x + Pt(fontsize);
                    char_y = loc_y_pt + content.space_y_pt * (count - 1) as f32;                    
                    ops.append(&mut add_text(font_backup_id, 
                                    content.pun_font_size_pt, 
                                    char_x, char_y, 
                                    &char.to_string(), 
                                    0.0));
                }
                3 =>{// 旋转字符
                    char_x = loc_x_pt + offset_x;
                    char_y = loc_y_pt + content.space_y_pt * count as f32 + Pt(fontsize * PUN_PUB);                    
                    ops.append(&mut add_text(font_backup_id, 
                                    fontsize, 
                                    char_x, char_y, 
                                    &char.to_string(), 
                                    -90.0));
                    count += 1;   

                }
                _ => {// 正常字符
                    char_x = loc_x_pt + offset_x;
                    char_y = loc_y_pt + content.space_y_pt * count as f32;                    
                    ops.append(&mut add_text(font_id, 
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


fn add_template(template: &Template, column_count: usize,bg_layer_id: LayerInternalId)->Vec<Op>{
    
    let mut ops = vec![];
    let bg_color = color_to_rgb(&template.bgcolor.clone().to_string());
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
                            x: Mm(template.page_width_mm).into_pt(),
                            y: Pt(0.0), // Top right
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Mm(template.page_width_mm).into_pt(),
                            y: Mm(template.page_height_mm).into_pt(), // Bottom right
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(0.0),
                            y: Mm(template.page_height_mm).into_pt(), // Bottom left
                        },
                        bezier: false,
                    },
                ],
            }],
            mode: printpdf::PaintMode::Fill,
            winding_order: printpdf::WindingOrder::NonZero,
        },
    });

    // 绘制页面外边框  
    let line_color = color_to_rgb(&template.linecolor.clone().to_string());   
    ops.push(Op::SetOutlineColor { col: line_color.clone() });
    ops.push(Op::SetOutlineThickness { pt: template.canvas.line_width_pt });
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: Point {
                        x: template.canvas.point_left_bottom.x - template.canvas.line_offset_pt,
                        y: template.canvas.point_left_bottom.y - template.canvas.line_offset_pt,
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: template.canvas.point_left_top.x - template.canvas.line_offset_pt,
                        y: template.canvas.point_left_top.y + template.canvas.line_offset_pt,
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: template.canvas.point_right_top.x + template.canvas.line_offset_pt,
                        y: template.canvas.point_right_top.y + template.canvas.line_offset_pt,
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: template.canvas.point_right_bottom.x + template.canvas.line_offset_pt,
                        y: template.canvas.point_right_bottom.y - template.canvas.line_offset_pt,
                    },
                    bezier: false,
                },
            ],
            is_closed: true,
        } } ); 
    //绘制鱼尾中线
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: Point {
                        x: Mm(template.page_width_mm / 2.0).into_pt(),
                        y: template.canvas.point_center_left_bottom.y, // Bottom left
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Mm(template.page_width_mm / 2.0).into_pt(),
                        y: template.tail.point_line_down_left.y, 
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
                        x: Mm(template.page_width_mm / 2.0).into_pt(),
                        y: template.tail.point_line_up_left.y, 
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Mm(template.page_width_mm / 2.0).into_pt(),
                        y: template.canvas.point_center_left_top.y, 
                    },  
                    bezier: false,
                },
            ],
            is_closed: false,
        } } );
    // 绘制页面内边框
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: template.canvas.point_left_bottom,
                    bezier: false,
                },
                LinePoint {
                    p: template.canvas.point_left_top,
                    bezier: false,
                },
                LinePoint {
                    p: template.canvas.point_right_top,
                    bezier: false,
                },
                LinePoint {
                    p: template.canvas.point_right_bottom,
                    bezier: false,
                },
            ],
            is_closed: true,
        } } ); 
    
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: template.canvas.point_center_left_bottom,
                    bezier: false,
                },
                LinePoint {
                    p: template.canvas.point_center_left_top,
                    bezier: false,
                },
                LinePoint {
                    p: template.canvas.point_center_right_top,
                    bezier: false,
                },
                LinePoint {
                    p: template.canvas.point_center_right_bottom,
                    bezier: false,
                },
            ],
            is_closed: true,
        } } ); 

    for i in 0..column_count{
        if i < column_count /2 {
            ops.push(Op::DrawLine { 
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point {
                                x: template.canvas.point_left_bottom.x 
                                   + template.canvas.column_width_pt * i as f32,
                                y: template.canvas.point_left_bottom.y, // Bottom left
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: template.canvas.point_left_top.x + template.canvas.column_width_pt * i as f32,
                                y: template.canvas.point_left_top.y, // Top left
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
                                x: template.canvas.point_right_bottom.x 
                                   - template.canvas.column_width_pt 
                                   * (i - column_count / 2) as f32,
                                y: template.canvas.point_right_bottom.y, // Bottom left
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: template.canvas.point_right_top.x 
                                   - template.canvas.column_width_pt 
                                   * (i - column_count / 2) as f32,
                                y: template.canvas.point_right_top.y, // Top left
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
                    p: template.tail.point_line_down_left,
                    bezier: false,
                },
                LinePoint {
                    p: template.tail.point_line_down_right,
                    bezier: false,
                },
            ],
            is_closed: false,
        } } ); 
    ops.push(Op::DrawLine { 
        line: Line {
            points: vec![
                LinePoint {
                    p: template.tail.point_line_up_left,
                    bezier: false,
                },
                LinePoint {
                    p: template.tail.point_line_up_right,
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
                        p: template.tail.point_up_left_top,
                        bezier: false,
                    },
                    LinePoint {
                        p: template.tail.point_up_right_top,  
                        bezier: false,
                    },
                    LinePoint {
                        p: template.tail.point_up_right_bottom,
                        bezier: false,
                    },
                    LinePoint {
                        p: template.tail.point_up_center,
                        bezier: false,
                    },
                    LinePoint {
                        p: template.tail.point_up_left_bottom,
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
                        p: template.tail.point_down_left_bottom,
                        bezier: false,
                    },
                    LinePoint {
                        p: template.tail.point_down_right_bottom,  
                        bezier: false,
                    },
                    LinePoint {
                        p: template.tail.point_down_right_top,
                        bezier: false,
                    },
                    LinePoint {
                        p: template.tail.point_down_center,
                        bezier: false,
                    },
                    LinePoint {
                        p: template.tail.point_down_left_top,
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
