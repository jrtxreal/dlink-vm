#![no_std]
#![no_main]

// Panic handler for no_std environment
#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// 导入宿主提供的函数
#[link(wasm_import_module = "dlinkwm_host")]
unsafe extern "C" {
    // 通用调用接口
    fn universal_invoke(
        method_name_ptr: i32,
        method_name_len: i32,
        format_type: i32,
        params_ptr: i32,
        params_len: i32,
        ret_ptr: i32,
    ) -> i32;
    
    // 内存分配函数
    fn host_malloc(size: i32) -> i32;
    
    // 内存释放函数
    fn host_free(ptr: i32);
}

// 主入口函数
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlinkwm_print_hello_wasm() -> i32 {
    // 返回一个静态字符串的指针
    b"hello wasm!\0" as *const u8 as i32
}

// 测试调用宿主自定义方法
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlinkwm_call_host_method() -> i32 {
    // 准备返回缓冲区大小
    let ret_buffer_size = 1024; // 1KB 足够存储返回数据
    
    // 使用宿主提供的内存分配函数分配返回缓冲区
    let ret_ptr = host_malloc(ret_buffer_size);
    if ret_ptr == 0 {
        return -1; // 内存分配失败
    }
    
    // -------------------------- 调用 custom_greet 方法 --------------------------
    let greet_method = b"custom_greet\0";
    let greet_params = b"{\"data\":{\"name\":\"WASM\"}}\0";
    
    let status = universal_invoke(
        greet_method.as_ptr() as i32,
        (greet_method.len() - 1) as i32,
        0, // JSON格式
        greet_params.as_ptr() as i32,
        (greet_params.len() - 1) as i32,
        ret_ptr
    );
    
    if status != 0 {
        host_free(ret_ptr); // 释放已分配内存
        return status; // 返回错误码
    }
    
    // 解析返回结果
    let response_status = core::ptr::read_unaligned((ret_ptr as *const u32).offset(0));
    let response_len = core::ptr::read_unaligned((ret_ptr as *const u32).offset(1));
    let response_data = ret_ptr + 8; // 跳过状态码和长度字段
    
    if response_status == 1 { // 成功
        // 创建一个新的缓冲区来存储带null终止符的字符串
        let result_ptr = host_malloc((response_len as i32) + 1);
        if result_ptr == 0 {
            host_free(ret_ptr);
            return -1;
        }
        
        // 复制响应数据到新缓冲区
        core::ptr::copy_nonoverlapping(
            response_data as *const u8,
            result_ptr as *mut u8,
            response_len as usize
        );
        
        // 添加null终止符
        core::ptr::write((result_ptr as *mut u8).offset(response_len as isize), 0u8);
        
        // 释放原始响应缓冲区
        host_free(ret_ptr);
        
        // 返回带null终止符的字符串指针
        result_ptr
    } else {
        host_free(ret_ptr);
        -1 // 调用失败
    }
}