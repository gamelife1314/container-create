use once_cell::sync::OnceCell;
use std::env;
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::os::unix::fs;
use std::process;
#[cfg(target_os = "linux")]
use std::ptr;

static HOSTNAME: OnceCell<String> = OnceCell::new();

extern "C" fn container_main(_args: *mut libc::c_void) -> libc::c_int {
    println!("Container - inside the container, pid: {}", process::id());

    // 切换进程的根目录
    #[cfg(target_os = "linux")]
    fs::chroot("./rootfs").expect("chroot failed");
    env::set_current_dir("/").expect("set current work directory failed");
    println!("current dir: {:?}", env::current_dir().unwrap());

    // 设置hostname
    hostname::set(HOSTNAME.get().unwrap()).expect("set container hostname failed");
    println!("current container hostname: {:?}", hostname::get().unwrap());

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
    println!(
        "Parent - start a container, pid: {}, hostname: {:?}",
        process::id(),
        hostname::get().expect("get parent hostname failed")
    );
    let mut stack = vec![0u8; 4096 * 1024];
    let args = env::args().collect::<Vec<String>>();
    let mut hostname = "container-default";
    if args.len() > 1 {
        hostname = &args[1];
    }
    HOSTNAME
        .set(hostname.to_string())
        .expect("init global var hostname failed");

    #[cfg(target_os = "linux")]
    unsafe {
        let stack_top = stack.as_mut_ptr().add(stack.len()) as *mut libc::c_void;
        let flags = libc::CLONE_NEWUTS
            | libc::CLONE_NEWNET
            | libc::CLONE_NEWPID
            | libc::CLONE_NEWNS
            | libc::SIGCHLD;
        let pid = libc::clone(container_main, stack_top, flags, ptr::null_mut());
        libc::waitpid(pid, ptr::null::<libc::c_int>() as *mut libc::c_int, 0);
    }
    println!("Parent - container stopped!");
}
