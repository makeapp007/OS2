use std::io;
use std::io::Write;
extern crate libc;
use libc::{system,fork,execvp,waitpid};
use libc::c_char;
use std::ffi::{CStr,CString};
use std::str;
use std::env;
use std::path::Path;
use std::process;


// #[derive(Debug)]
fn exec_pwd(){
    let p = env::current_dir().unwrap();
    // p+1
    println!("{}", p.display());
  
}
fn exec_cd(x:&String ){
    let path=Path::new(&x);
    let ret_val=env::set_current_dir(&path).is_ok();
    if ! ret_val{
        println!("error occurs in cd ");
    }

}
fn print_history(history:&Vec<String> ){
    // let length=history.len();
    let mut index=1;
    for x in history   {
        // println!("{}", x);
        println!("{:>5}  {}", index, x);
        index=index+1;
    }
}
fn start_dash(){
    let mut input = String::new();
    let mut history_store: Vec<String> =Vec::new();

    loop{
        input = String::new();
        print!("$ ");
        io::stdout().flush();

        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                // println!("{} bytes read", n);
                input.pop(); //pop \n                
            }
            Err(error) => println!("error: {}", error),
        }


        // parse the input
        let input_ele: Vec<String> = input.split_whitespace().map(|x| x.to_string()).collect();

        // execute
        if input_ele.len()==0{
            history_store.push(input.clone());
            continue;
        }
        else{
            if input_ele[0]=="pwd"{
                exec_pwd();
            }
            else if input_ele[0]=="cd"{
                if input_ele.len()==2 {
                    exec_cd(&input_ele[1]);
                }
                else{
                    println!("Wrong parameters");
                }
            }
            else if input_ele[0]=="exit"{
                process::exit(0);
            }
            else if input_ele[0]=="history"{
                print_history(& history_store);
            }
            else{
                // external command
                // fork and execvp
                unsafe{
                    let fork_result=fork();
                    if fork_result< 0 {
                        println!("Fail to fork");

                    }
                    else if fork_result==0{
                        // in child's process
                        let command=input_ele[0].clone();
                        let c_command=CString::new(command).unwrap();
                        // input_ele.iter().next();
                        // input_ele.iter().next() ;
                        // println!("------left is {:?}",input_ele.iter().next() );

                        // fetch the correct data?
                        let cstr_argv: Vec<_> = input_ele.iter()
                                .map(|arg| CString::new(arg.as_str()).unwrap())
                                .collect();
                        // for i in &cstr_argv{
                        //     println!("-----{:?}", i);
                        // }

                        let mut p_argv: Vec<_> = cstr_argv.iter()
                                .map(|arg| arg.as_ptr())
                                .collect();
                        // for i in &p_argv{
                        //     println!("====={:?}", i);
                        // }

                        p_argv.push(std::ptr::null());
                        let p: *const *const c_char = p_argv.as_ptr();
                        let ret_val=execvp(c_command.as_ptr(),p);
                        // println!("ret_val is {}",ret_val);
                    }
                    else {
                        // in parents' process
                        let mut my_num: i32 = 10;
                        let status: *mut i32 = &mut my_num;
                        let ret_val=waitpid(fork_result,status,0);

                    }
                }
            }
            history_store.push(input.clone());
            
        }

        // store it to history
        
// pub unsafe extern fn execvp(c: *const c_char,
//                             argv: *const *const c_char)
//                             -> c_int
// pub unsafe extern fn waitpid(pid: pid_t,
//                              status: *mut c_int,
//                              options: c_int)
//                              -> pid_t



        // pub unsafe extern fn system(s: *const c_char) -> c_int
        // let ptr = CString::new("ls").unwrap().as_ptr();
        // input_ele[0]+1;
        // let raw = b"foo".to_vec();
        // let command=input_ele[0].to_vec();
        // let command1=CString::from_vec_unchecked(command);
        // println!("the input is {}",input);
        // let command2=CString::new(input);
        // let command=command2.unwrap().as_ptr();
        // unsafe{
        //     // println!("command is {} ",command.display());
        //     // system(command);
        //     // // CString::new("Hello")
            
        //     // system(CString::new("ls").unwrap().as_ptr());
        //     let ret_val=system( command);   
        //     // println!("return value is {}",ret_val);    
        //     // match ret_val{
        //     //     0 => println!("no command processor is available"),
        //     //     -1 => println!("it wasn’t possible to create the shell process"),
        //     //     _ => println!("return value is{}, a command has been executed ",ret_val),
        //     // }
        // }
        // println!("the input is{} ",input);
        // println!("this is input: {:?},{}",input_ele,input_ele.len());

    // let params: Vec<String> = line.split(" ").map(|x| x.to_string()).collect();
    // let mut iter = params.into_iter();

    // let cmd = iter.next().unwrap();
    // let rest: Vec<String> = iter.collect();


        // input.pop();
        // print!("{}", (8u8 as char));
    }    
    // let mut input = String::new();
    // loop {
    //     print!("$ ");
    //     let line = io::stdin().read_line(&mut input);
    //     match line {
    //         Ok(expr) => println!("{}", expr),
    //         Err(_) => break,
    //     }
    // }

}



fn main() {
    // println!("below is the output");
    // println!("{number:>width$}", number=1, width=6);

    start_dash();
}

