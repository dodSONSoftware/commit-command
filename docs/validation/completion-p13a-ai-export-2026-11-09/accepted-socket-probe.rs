use std::{io::{Read,Write},net::{TcpListener,TcpStream},os::fd::AsRawFd,time::Duration};
unsafe extern "C" { fn fcntl(fd:i32, cmd:i32, ...) -> i32; }
fn main() {
 let listener=TcpListener::bind("127.0.0.1:0").unwrap(); listener.set_nonblocking(true).unwrap();
 let mut client=TcpStream::connect(listener.local_addr().unwrap()).unwrap();
 let (mut accepted,_)=listener.accept().unwrap(); accepted.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
 let before=unsafe{fcntl(accepted.as_raw_fd(),3)}; println!("accepted_O_NONBLOCK={}",before&4);
 let mut byte=[0]; let start=std::time::Instant::now(); let result=accepted.read(&mut byte); println!("before_read={:?}; elapsed_us={}",result.as_ref().err().map(|e|e.kind()),start.elapsed().as_micros());
 assert_ne!(before&4,0); assert_eq!(result.unwrap_err().kind(),std::io::ErrorKind::WouldBlock);
 accepted.set_nonblocking(false).unwrap(); let after=unsafe{fcntl(accepted.as_raw_fd(),3)}; println!("after_O_NONBLOCK={}",after&4); assert_eq!(after&4,0);
 let writer=std::thread::spawn(move||{ std::thread::sleep(Duration::from_millis(25)); client.write_all(b"x").unwrap(); });
 assert_eq!(accepted.read(&mut byte).unwrap(),1); assert_eq!(&byte,b"x"); writer.join().unwrap(); println!("delayed_request_byte=passed");
}
