#[cfg(feature = "sqlite")]
pub mod sqlite {
    use serde_json::Value;
    use std::ptr::null;
    use std::ffi::CString;
    use std::ffi::CStr;
    use lib_sqlite_bingen::*;

    pub fn execute_sqlite_query(conninfo: String, query: String, params: Value) -> Value{
        let mut db: *mut sqlite3 = std::ptr::null_mut();
        let mut statement_result: *mut sqlite3_stmt = std::ptr::null_mut();
        unsafe {
            let rc = lib_sqlite_bingen::sqlite3_open_v2(
                CString::new(String::from(conninfo)).unwrap().as_ptr(),
                &mut db,
                (SQLITE_OPEN_CREATE as i32) | (SQLITE_OPEN_READWRITE as i32),
                null()
            );
    
            let mut rc = lib_sqlite_bingen::sqlite3_prepare(
                db,
                CString::new(String::from(query)).unwrap().as_ptr(),
                -1,
                &mut statement_result,
                std::ptr::null_mut()
            );
            if !params.is_array() {
                panic!("Value must be array");
            }
            let param_array = params.as_array().unwrap();
            if (rc as u32) == SQLITE_OK {
                let mut entry_counter = 0;
                for entry in param_array {
                    if entry.is_string() {
                        sqlite3_bind_text(
                            statement_result,
                            entry_counter + 1,
                            CString::new(entry.as_str().unwrap()).unwrap().as_ptr(),
                            -1,
                            Some(std::mem::transmute(-1_isize))
                        );
                    }
    
                    if entry.is_boolean() {
                        sqlite3_bind_int(
                            statement_result,
                            entry_counter + 1,
                            entry.as_bool().unwrap() as i32
                        );
                    }
                    if entry.is_number() {
                        sqlite3_bind_double(
                            statement_result,
                            entry_counter + 1,
                            entry.as_f64().unwrap()
                        );
                    } else if entry.is_array() {
                        sqlite3_bind_text(
                            statement_result,
                            entry_counter + 1,
                            CString::new(serde_json::to_string(&entry).unwrap()).unwrap().as_ptr(),
                            entry.as_str().unwrap().len() as i32,
                            Some(std::mem::transmute(-1_isize))
                        );
                    } else if entry.is_object() {
                        sqlite3_bind_text(
                            statement_result,
                            entry_counter + 1,
                            CString::new(serde_json::to_string(&entry).unwrap()).unwrap().as_ptr(),
                            entry.as_str().unwrap().len() as i32,
                            Some(std::mem::transmute(-1_isize))
                        );
                    } else if entry.is_null() {
                        sqlite3_bind_null(statement_result, 0);
                    }
                    entry_counter = entry_counter + 1;
                }
            } else {
                println!("{}",String::from(CStr::from_ptr(sqlite3_errmsg(db)).to_str().unwrap()));
                sqlite3_close(db);
                return serde_json::json!([]);
            }
    
            let mut rc = sqlite3_step(statement_result);
    
            if rc == (SQLITE_ROW as i32) {
                let mut sqlite_result_vec:Vec<std::collections::BTreeMap<String,Value>> = Vec::new();
                loop {
                    let mut col_count = 0;
                    let mut row_result: std::collections::BTreeMap<
                        String,
                        Value
                    > = std::collections::BTreeMap::new();
                    let col_nums = sqlite3_column_count(statement_result);
                    while col_count <  col_nums{
                        let col_name = String::from(CStr::from_ptr(sqlite3_column_name(statement_result,col_count)).to_str().unwrap());
                        let column_result = String::from(CStr::from_ptr(sqlite3_column_text(statement_result, col_count) as *const i8).to_str().unwrap());
                        let num = column_result.parse::<f64>();
                        if num.is_ok(){
                            let num_parsed:f64 = num.unwrap();
                            col_count += 1;
                            let num_val = serde_json::json!(num_parsed);
                            row_result.insert(
                                col_name, 
                                num_val
                            );
                            continue;
                        }
                        let num = column_result.parse::<i64>();
                        if num.is_ok(){
                            let num_parsed:i64 = num.unwrap();
                            col_count += 1;
                            let num_val = serde_json::json!(num_parsed);
                            row_result.insert(
                                col_name, 
                                num_val
                            );
                            continue;
                        }
                        let bool_val = column_result.parse::<bool>();
                        if bool_val.is_ok(){
                            let bool_val:bool = bool_val.unwrap();
                            col_count += 1;
                            let bool_val = serde_json::json!(bool_val);
                            row_result.insert(
                                col_name, 
                                bool_val
                            );
                            continue;
                        }

                        let string_val = column_result.parse::<String>();
                        if string_val.is_ok(){
                            let string_val:String = string_val.unwrap();
                            col_count += 1;
                            let string_val = serde_json::json!(string_val);
                            row_result.insert(
                                col_name, 
                                string_val
                            );
                            continue;
                        }
                    }
                    sqlite_result_vec.push(row_result);
                    rc = sqlite3_step(statement_result);
                    if rc != (SQLITE_ROW as i32){
                        break;
                    }
                }
                sqlite3_close(db);
                return serde_json::json!(sqlite_result_vec);
            } else {
                lib_sqlite_bingen::sqlite3_finalize(statement_result);
                sqlite3_close(db);
                return serde_json::json!([]);
            }
            
            
        }
    }
}
