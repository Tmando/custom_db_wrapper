
#[cfg(feature = "postgres")]
pub mod postgres{
    use lib_postgres_bindgen::*;
    use serde_json::Value;
    use serde_json::json;
    use core::ffi::c_void;
    use std::ffi::CString;
    use std::ffi::CStr;
    use std::ptr::null;
    fn parse_postgres_array(mut input_string:String)->String{
        println!("{:?}", input_string);
        let mut curPos = 0;
        while curPos < input_string.len(){
            let mut before_char = '\0';
            let mut cur_char = input_string.chars().nth(curPos).unwrap();
            let mut after_char = '\0';
            if curPos as i32 - 1 >= 0 {
                before_char = input_string.chars().nth(curPos).unwrap();
            }
            if curPos + 1 <  input_string.len() {
                after_char = input_string.chars().nth(curPos).unwrap();
            }
            if cur_char == '{' && before_char != '\''{
                input_string.replace_range(curPos..curPos+1,"[");
            }
            if cur_char == '}' && before_char != '\''{
                input_string.replace_range(curPos..curPos+1,"]");
            }
            curPos += 1;
        }
        println!("{}",input_string);
        String::from(input_string)
    }
    fn parse_postgres_result_json(res: *mut pg_result)-> Vec<std::collections::BTreeMap<String, Value>>{
            unsafe{
                let rows = lib_postgres_bindgen::PQntuples(res);
                let fields = lib_postgres_bindgen::PQnfields(res);
                let mut field_vec: Vec<String> = Vec::new();
                let mut query_result_map: Vec<std::collections::BTreeMap<String, Value>> = Vec::new();
                for i in 0..fields {
                    field_vec.push(
                        String::from(
                            CStr::from_ptr(lib_postgres_bindgen::PQfname(res, i)).to_str().unwrap()
                        )
                    );
                }
    
                for n in 0..rows {
                    let mut row_result: std::collections::BTreeMap<
                        String,
                        Value
                    > = std::collections::BTreeMap::new();
                    for j in 0..fields {
                        let field_res = String::from(
                            CStr::from_ptr(lib_postgres_bindgen::PQgetvalue(res, n, j))
                                .to_str()
                                .unwrap()
                        );
                        let mut json_res:Value = Value::Null;
                        if field_res != "" && field_res != "null"{
                            json_res = match PQftype(res,j){
                                /* Int Types */
                                20 | 21 | 23 =>{
                                    json!(field_res.parse::<i64>().unwrap())
                                },
                                /* Int Array Types */
                                1231 | 1005 | 1007 | 1016 | 1021| 1022 => {
                                    json!(
                                        parse_postgres_array(field_res).parse::<Value>().unwrap())
                                },
                                /* Float Types */
                                1700 | 700 | 701 => {
                                    json!(field_res.parse::<f64>().unwrap())
                                },
                                /* Money Types */
                                790 =>{
                                    json!(field_res.parse::<String>().unwrap())
                                },
                                791 =>{
                                    json!(parse_postgres_array(field_res).parse::<String>().unwrap())
                                },
                                /* String Types */
                                25 | 1042 | 1043 =>{
                                    json!(field_res.parse::<String>().unwrap())
                                },
                                /* Boolean Type */
                                16 => {
                                    if field_res == "t"{
                                        json!(true)
                                    }else if field_res == "f"{
                                        json!(false)
                                    }else{
                                        json!(String::from(""))
                                    }
                                },
                                /* Bin Types */
                                17 =>{
                                    json!(field_res.parse::<String>().unwrap())
                                },
                                /* Date Types */
                                1082 | 1114 | 1083 =>{
                                    json!(field_res.parse::<String>().unwrap())
                                },
                                /* Geo Types */
                                600 | 602 | 601 | 604 | 628 | 603 | 718 =>{
                                    json!(field_res.parse::<String>().unwrap())
                                },
                                /* inet types */
                                869 | 650 | 829 | 774 => {
                                    json!(field_res.parse::<String>().unwrap())
                                }
                                /* bit types */
                                1560 | 1562 => {
                                    json!(field_res.parse::<String>().unwrap())
                                },
                                /* tsvector */
                                3614 | 3615 => {
                                    json!(field_res.parse::<String>().unwrap())
                                },
                                /* uuid */
                                2950 => {
                                    json!(field_res.parse::<String>().unwrap())
                                },
                                142 => {
                                    json!(field_res.parse::<String>().unwrap())
                                }
                                114 | 3802 => {
                                    json!(field_res.parse::<Value>().unwrap())
                                }
                                _ =>{
                                    json!(field_res)
                                }
                            };
                        }
                        row_result.insert(String::from(field_vec.get(j as usize).unwrap()), json_res);
                    }
                    query_result_map.push(row_result);
                }
                return query_result_map;
            }
    }

    pub fn execute_postgres_query_non_blocking_with_parameters(
        conninfo: String,
        query: String,
        params: Value,
        timeout: i32
    )-> Value{
        unsafe {
            let connection = lib_postgres_bindgen::PQconnectStart(
                CString::new(String::from(conninfo)).unwrap().as_ptr()
            );
    
            if
                lib_postgres_bindgen::PQstatus(connection) ==
                lib_postgres_bindgen::ConnStatusType_CONNECTION_BAD
            {
                println!(
                    "Connection to database failed: {:?}",
                    CStr::from_ptr(lib_postgres_bindgen::PQerrorMessage(connection))
                );
                lib_postgres_bindgen::PQfinish(connection);
                return serde_json::Value::Null;
            }

            let start = std::time::Instant::now();
            let mut out_of_timeout = false;
            
            while
                lib_postgres_bindgen::PQconnectPoll(connection) !=
                lib_postgres_bindgen::PostgresPollingStatusType_PGRES_POLLING_OK
            {
                let status = lib_postgres_bindgen::PQconnectPoll(connection);
                if status == PostgresPollingStatusType_PGRES_POLLING_FAILED {
                    println!(
                        "Connection to database failed: {:?}",
                        CStr::from_ptr(lib_postgres_bindgen::PQerrorMessage(connection))
                    );
                }
                let duration = start.elapsed();
                
                
                if duration.as_millis() as i32 > timeout {
                    out_of_timeout = true;
                    break;
                }
            }


            if out_of_timeout == true {
                println!("{:?}",String::from("Out of timeout!"));
                return serde_json::Value::Null;
            }

            if !params.is_array() {
                lib_postgres_bindgen::PQfinish(connection);
                panic!("Value must be array");
            }

            let param_value_array = params.as_array().unwrap();
            let mut cs_string_vec: Vec<std::ffi::CString> = Vec::new();
    
            for entry in param_value_array {
                if entry.is_string() {
                    cs_string_vec.push(CString::new(entry.as_str().unwrap()).unwrap());
                }
    
                if entry.is_boolean() {
                    cs_string_vec.push(CString::new(entry.as_bool().unwrap().to_string()).unwrap());
                }
    
                if entry.is_number() {
                    cs_string_vec.push(CString::new(entry.as_number().unwrap().to_string()).unwrap());
                }
    
                if entry.is_array() {
                    cs_string_vec.push(CString::new(serde_json::to_string(&entry).unwrap()).unwrap());
                }
    
                if entry.is_object() {
                    cs_string_vec.push(CString::new(serde_json::to_string(&entry).unwrap()).unwrap());
                }
                if entry.is_null() {
                    cs_string_vec.push(CString::new(serde_json::to_string(&entry).unwrap()).unwrap());
                }
            }
            
    
            let mut p_argv: Vec<_> = cs_string_vec
                .iter() // do NOT into_iter()
                .map(|arg| arg.as_ptr())
                .collect();
    
            p_argv.push(null());
    
            let res = lib_postgres_bindgen::PQexecParams(
                connection, // param 1
                CString::new(String::from(&query)).unwrap().as_ptr(), // param 2
                (p_argv.len() - 1) as i32, // param 3
                null(), // param 4
                p_argv.as_ptr() as *const *const ::std::os::raw::c_char, // param 5
                null(), // param 6
                null(), // param 7
                0 as i32 // param 8
            );
    
            if
                lib_postgres_bindgen::PQresultStatus(res) !=
                    lib_postgres_bindgen::ExecStatusType_PGRES_COMMAND_OK &&
                lib_postgres_bindgen::PQresultStatus(res) !=
                    lib_postgres_bindgen::ExecStatusType_PGRES_TUPLES_OK
            {
                println!("{:?}", CStr::from_ptr(lib_postgres_bindgen::PQerrorMessage(connection)));
                lib_postgres_bindgen::PQclear(res);
                lib_postgres_bindgen::PQfinish(connection);
                return serde_json::Value::Null;
            }
    
            if
                lib_postgres_bindgen::PQresultStatus(res) ==
                lib_postgres_bindgen::ExecStatusType_PGRES_COMMAND_OK
            {
                lib_postgres_bindgen::PQclear(res);
                lib_postgres_bindgen::PQfinish(connection);
                return serde_json::json!([]);
            }
    
            if
                lib_postgres_bindgen::PQresultStatus(res) ==
                lib_postgres_bindgen::ExecStatusType_PGRES_TUPLES_OK
            {
                let result_map = parse_postgres_result_json(res);
                lib_postgres_bindgen::PQclear(res);
                lib_postgres_bindgen::PQfinish(connection);
                return json!(result_map);
            }
            lib_postgres_bindgen::PQclear(res);
            lib_postgres_bindgen::PQfinish(connection);
            return serde_json::json!([]);
        }
    }
    
    pub fn execute_postgres_query_non_blocking(conninfo: String, query: String)-> Value {
        unsafe {
            let connection = lib_postgres_bindgen::PQconnectStart(
                CString::new(String::from(conninfo)).unwrap().as_ptr()
            );
            if
                lib_postgres_bindgen::PQstatus(connection) ==
                lib_postgres_bindgen::ConnStatusType_CONNECTION_BAD
            {
                println!(
                    "Connection to database failed: {:?}",
                    CStr::from_ptr(lib_postgres_bindgen::PQerrorMessage(connection))
                );
                lib_postgres_bindgen::PQfinish(connection);
                return serde_json::Value::Null;
            }
            while
                lib_postgres_bindgen::PQconnectPoll(connection) !=
                lib_postgres_bindgen::PostgresPollingStatusType_PGRES_POLLING_OK
            {
                let status = lib_postgres_bindgen::PQconnectPoll(connection);
                if status == PostgresPollingStatusType_PGRES_POLLING_FAILED {
                    println!(
                        "Connection to database failed: {:?}",
                        CStr::from_ptr(lib_postgres_bindgen::PQerrorMessage(connection))
                    );
                }
            }
            let res = lib_postgres_bindgen::PQexec(
                connection,
                CString::new(String::from(&query)).unwrap().as_ptr()
            );
            if
                lib_postgres_bindgen::PQresultStatus(res) !=
                    lib_postgres_bindgen::ExecStatusType_PGRES_COMMAND_OK &&
                lib_postgres_bindgen::PQresultStatus(res) !=
                    lib_postgres_bindgen::ExecStatusType_PGRES_TUPLES_OK
            {
                println!("{:?}", CStr::from_ptr(lib_postgres_bindgen::PQerrorMessage(connection)));
                lib_postgres_bindgen::PQfinish(connection);
                return serde_json::Value::Null;
            }
    
            if
                lib_postgres_bindgen::PQresultStatus(res) ==
                lib_postgres_bindgen::ExecStatusType_PGRES_COMMAND_OK
            {
                lib_postgres_bindgen::PQclear(res);
                lib_postgres_bindgen::PQfinish(connection);
                return json!([]);
            }
    
            if
                lib_postgres_bindgen::PQresultStatus(res) ==
                lib_postgres_bindgen::ExecStatusType_PGRES_TUPLES_OK
            {
                let result_map = parse_postgres_result_json(res);
                lib_postgres_bindgen::PQclear(res);
                lib_postgres_bindgen::PQfinish(connection);
                return json!(result_map);
            }

            lib_postgres_bindgen::PQclear(res);
            lib_postgres_bindgen::PQfinish(connection);
            return json!([]);
        }
    }
    
    pub fn execute_postgres_query(conninfo: String, query: String) -> Value{
        unsafe {
            let connection = lib_postgres_bindgen::PQconnectdb(
                CString::new(String::from(conninfo)).unwrap().as_ptr()
            );
            if
                lib_postgres_bindgen::PQstatus(connection) ==
                lib_postgres_bindgen::ConnStatusType_CONNECTION_BAD
            {
                println!(
                    "Connection to database failed: {:?}",
                    CStr::from_ptr(lib_postgres_bindgen::PQerrorMessage(connection))
                );
                lib_postgres_bindgen::PQfinish(connection);
                return serde_json::Value::Null;
            }
            let res = lib_postgres_bindgen::PQexec(
                connection,
                CString::new(String::from(query)).unwrap().as_ptr()
            );
            if
                lib_postgres_bindgen::PQresultStatus(res) !=
                    lib_postgres_bindgen::ExecStatusType_PGRES_COMMAND_OK &&
                lib_postgres_bindgen::PQresultStatus(res) !=
                    lib_postgres_bindgen::ExecStatusType_PGRES_TUPLES_OK
            {
                println!("{:?}", CStr::from_ptr(lib_postgres_bindgen::PQerrorMessage(connection)));
                lib_postgres_bindgen::PQfinish(connection);
                return serde_json::Value::Null;
            }

            if
                lib_postgres_bindgen::PQresultStatus(res) ==
                lib_postgres_bindgen::ExecStatusType_PGRES_COMMAND_OK
            {
                return json!([]);
            }

            if
                lib_postgres_bindgen::PQresultStatus(res) ==
                lib_postgres_bindgen::ExecStatusType_PGRES_TUPLES_OK
            {
                let result_map = parse_postgres_result_json(res);
                lib_postgres_bindgen::PQclear(res);
                lib_postgres_bindgen::PQfinish(connection);
                return json!(result_map);
            }
            lib_postgres_bindgen::PQclear(res);
            lib_postgres_bindgen::PQfinish(connection);
            return json!([]);
        }
    }
}