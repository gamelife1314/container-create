use std::env;
use std::ffi::CString;
use std::os::unix::fs;
use std::process;
#[cfg(target_os = "linux")]
use std::ptr;

extern "C" fn container_main(_args: *mut libc::c_void) -> libc::c_int {
    println!("Container - inside the container, pid: {}", process::id());

    // 切换进程的根目录
    fs::chroot("./rootfs").expect("chroot failed");
    env::set_current_dir("/").expect("set current work directory failed");

    // 打印当前目录
    println!("current dir: {:?}", env::current_dir().unwrap());

    #[cfg(target_os = "linux")]
    unsafe {
        // 挂载 proc
        let source = CString::new("none").unwrap();
        let target = CString::new("/proc").unwrap();
        let fstype = CString::new("proc").unwrap();
        let flags = 0 as libc::c_ulong;
        let data = CString::new("").unwrap();
        let result = libc::mount(
            source.as_ptr(),
            target.as_ptr(),
            fstype.as_ptr(),
            flags,
            data.as_ptr() as *const libc::c_void,
        );
        println!("mount result: {}", result);

        // 运行进程
        let program = CString::new("/bin/bash").unwrap();
        let args = [program.as_ptr(), ptr::null()];
        libc::execvp(program.as_ptr(), args.as_ptr());
    }
    println!("Container - Something Wrong");
    return 1;
}

fn main() {
    println!("Parent - start a container, pid: {}", process::id());
    let mut stack = vec![0u8; 4096 * 1024];
    #[cfg(target_os = "linux")]
    unsafe {
        let stack_top = stack.as_mut_ptr().add(stack.len()) as *mut libc::c_void;
        let flags = libc::CLONE_NEWNET | libc::CLONE_NEWPID | libc::CLONE_NEWNS | libc::SIGCHLD;
        let pid = libc::clone(container_main, stack_top, flags, ptr::null_mut());
        libc::waitpid(pid, ptr::null::<libc::c_int>() as *mut libc::c_int, 0);
    }
    println!("Parent - container stopped!");
}
