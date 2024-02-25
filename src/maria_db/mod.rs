#[cfg(feature = "maria_db")]
pub mod maria_db {
    use lib_mysql_bindgen::*;
    use std::ptr::null;
    use std::ffi::CString;
    use std::ffi::CStr;
    use serde_json::Value;
    
    pub fn execute_maria_db(
        host: String,
        user: String,
        passwd: String,
        db: String,
        port: u32,
        query: String
    ) -> Value {
        unsafe {
            let connnection: *mut MYSQL = mysql_init(std::ptr::null_mut());
            loop {
                let status = mysql_real_connect_nonblocking(
                    connnection,
                    CString::new(host.clone()).unwrap().as_ptr(),
                    CString::new(user.clone()).unwrap().as_ptr(),
                    CString::new(passwd.clone()).unwrap().as_ptr(),
                    CString::new(db.clone()).unwrap().as_ptr(),
                    port,
                    std::ptr::null_mut(),
                    0
                );
                if status == 2 {
                    return Value::Null;
                }
                if status == 0 {
                    break;
                }
            }
            let query_res = mysql_query(connnection, CString::new(query.clone()).unwrap().as_ptr());
            if query_res == 1 {
                println!("{:?}", CStr::from_ptr(mysql_error(connnection)));
                mysql_close(connnection);
                return Value::Null;
            }
    
            let result: *mut MYSQL_RES = mysql_store_result(connnection);
            if(result == std::ptr::null_mut()){
                mysql_free_result(result);
                mysql_close(connnection);
                return Value::Null;
            }
            let result_ref = *result;
            let num_fields = mysql_num_fields(result);
            let fields = mysql_fetch_field(result);
            let mut mysql_field_vec: Vec<MYSQL_FIELD> = Vec::new();
            for i in 0..num_fields {
                let cur_field = *fields.add(i as usize);
                mysql_field_vec.push(cur_field);
            }
            if result_ref.row_count == 0 {
                return Value::Null;
            }
            let mut j = 0;
            // let mut i = 0;
            let mut query_result_map: Vec<std::collections::BTreeMap<String, Value>> = Vec::new();
            while j < result_ref.row_count {
                let row = mysql_fetch_row(result);
                let deref_row = *row;
                let mut offset: usize = 0;
                let lengths = mysql_fetch_lengths(result);
                let deref_len = lengths;
                let mut row_result: std::collections::BTreeMap<
                    String,
                    Value
                > = std::collections::BTreeMap::new();
                
                for k in 0..mysql_field_vec.len() {
                    let mut field_value_extracted;
                    // get field value with offset
                    if k > 0 {
                        field_value_extracted = CStr::from_ptr(deref_row.add(offset)).to_str().unwrap();
                    } else {
                        field_value_extracted = CStr::from_ptr(deref_row).to_str().unwrap();
                    }
                    
                    if field_value_extracted == ""{
                        let v = serde_json::json!(serde_json::Value::Null);
                        row_result.insert(String::from(CStr::from_ptr(mysql_field_vec[k].name).to_str().unwrap()), v);
                    }
                    
                    else if
                        [
                            enum_field_types_MYSQL_TYPE_DECIMAL,
                            enum_field_types_MYSQL_TYPE_FLOAT,
                            enum_field_types_MYSQL_TYPE_DOUBLE,
                            enum_field_types_MYSQL_TYPE_NEWDECIMAL
                        ].contains(&mysql_field_vec[k].type_)
                    {
                        let num = field_value_extracted.parse::<f64>().unwrap();
                        let v = serde_json::json!(num);
                        row_result.insert(String::from(CStr::from_ptr(mysql_field_vec[k].name).to_str().unwrap()), v);
                    }
    
                    else if
                        [
                            enum_field_types_MYSQL_TYPE_TINY,
                            enum_field_types_MYSQL_TYPE_SHORT,
                            enum_field_types_MYSQL_TYPE_LONG,
                            enum_field_types_MYSQL_TYPE_INT24,
                            enum_field_types_MYSQL_TYPE_LONGLONG
                        ].contains(&mysql_field_vec[k].type_)
                    {
                        let num = field_value_extracted.parse::<i64>().unwrap();
                        let v = serde_json::json!(num);
                        row_result.insert(String::from(CStr::from_ptr(mysql_field_vec[k].name).to_str().unwrap()), v);
                    }
                    
                    else if
                        [
                            enum_field_types_MYSQL_TYPE_TIMESTAMP,
                            enum_field_types_MYSQL_TYPE_DATE,
                            enum_field_types_MYSQL_TYPE_TIME,
                            enum_field_types_MYSQL_TYPE_DATETIME,
                            enum_field_types_MYSQL_TYPE_YEAR,
                            enum_field_types_MYSQL_TYPE_NEWDATE,
                            enum_field_types_MYSQL_TYPE_TIMESTAMP2,
                            enum_field_types_MYSQL_TYPE_DATETIME2,
                            enum_field_types_MYSQL_TYPE_TIME2
                        ].contains(&mysql_field_vec[k].type_)
                    {
                        let date = field_value_extracted.parse::<String>().unwrap();
                        let v = serde_json::json!(date);
                        row_result.insert(String::from(CStr::from_ptr(mysql_field_vec[k].name).to_str().unwrap()), v);
                    }
    
                    else if
                        [
                            enum_field_types_MYSQL_TYPE_VARCHAR,
                            enum_field_types_MYSQL_TYPE_VAR_STRING,
                            enum_field_types_MYSQL_TYPE_STRING,
                            enum_field_types_MYSQL_TYPE_JSON,
                            enum_field_types_MYSQL_TYPE_TINY_BLOB,
                            enum_field_types_MYSQL_TYPE_BLOB,
                            enum_field_types_MYSQL_TYPE_MEDIUM_BLOB,
                            enum_field_types_MYSQL_TYPE_LONG_BLOB
                        ].contains(&mysql_field_vec[k].type_)
                    {
                        let str_val = field_value_extracted.parse::<String>().unwrap();
                        let v = serde_json::json!(str_val);
                        row_result.insert(String::from(CStr::from_ptr(mysql_field_vec[k].name).to_str().unwrap()), v);
                    } else {
                        let str_val = field_value_extracted.parse::<String>().unwrap();
                        let v = serde_json::json!(str_val);
                        row_result.insert(String::from(CStr::from_ptr(mysql_field_vec[k].name).to_str().unwrap()), v);
                    }
                    offset = offset + (*lengths.add(k) as usize) + 1;
                }
                query_result_map.push(row_result);
                j = j + 1;
            }
            mysql_free_result(result);
            mysql_close(connnection);
            return serde_json::json!(query_result_map);
        }
    }
}