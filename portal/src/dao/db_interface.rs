use actix_web::{
    web::{self}, Responder,
};
use metapower_framework::DataResponse;

use crate::model::GroupServer;

pub async fn get_all_rows(
    path: web::Path<(String, String, String)>
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (token, session, table) = path.into_inner();

    Ok(web::Json(resp))
}
pub async fn get_row_by_id(
    path: web::Path<(String, String, String, String)>
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (token, session, table, id) = path.into_inner();

    Ok(web::Json(resp))
}

pub async fn save_table_rows(
    path: web::Path<(String, String, String)>, rows_jsonstr: web::Bytes
) -> actix_web::Result<impl Responder> {
    let mut resp = DataResponse {
        content: String::from(""),
        code: String::from("200"),
    };

    let (token, session, table) = path.into_inner();

    let buf = rows_jsonstr.to_vec();
    match std::str::from_utf8(&buf){
        Ok(json_str) => {
            match table.as_ref() {
                "groupserver" => {
                    match GroupServer::save_rows(json_str.to_owned()) {
                        Ok(_) => {
                            println!("保存数据成功!");
                        },
                        Err(e) => {
                            println!("保存数据失败:  {}", e);
                            resp.content = format!("保存数据失败: {}",e);
                            resp.code = String::from("500");
                            return Ok(web::Json(resp));
                        }
                    }
                },
                _ => {
                    println!("未知的表名: {}", table);
                    resp.content = format!("未知的表名: {}", table);
                    resp.code = String::from("500");
                    return Ok(web::Json(resp));
                }
            }
        }
        Err(e) => {
            resp.content = format!("参数错误{}",e);
            resp.code = String::from("500");
        }
    }

    Ok(web::Json(resp))
}