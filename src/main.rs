
use std::io;
use std::io::Write;
extern crate libc;
use libc::{system,fork,execvp,waitpid,wait,kill,pipe,dup2,close,open};
use libc::{O_RDONLY,O_WRONLY,O_CREAT,S_IRWXU,O_TRUNC};
use libc::c_char;
use std::ffi::{CStr,CString};
use std::str;
use std::env;
use std::path::Path;


//d #[derive(Debug)]
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
    let mut job_store: Vec<String> =Vec::new();

    loop{
        input = String::new();
        print!("$ ");
        io::stdout().flush();

        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                // println!("{} bytes read", n);
                if n ==0{
                    break
                }
                else{
                    input.pop(); //pop \n                
                }
            },
            // Ok(0) => break,
            Err(error) => println!("error: {}", error),
        }



        // parse the input
        let mut input_ele: Vec<String> = input.split_whitespace().map(|x| x.to_string()).collect();

        // counting how many pipes are there
        let mut pipe_count=0;
        for i in &input_ele{
            // autoderef
            if i =="|"{
                pipe_count=pipe_count+1;
            }
        }
        // println!("pipe_count is  {:?}",pipe_count );



        if pipe_count!=0{

            let mut v = vec![0; pipe_count*2]; // ten zeroes
            let v_slice=(&mut v[..]).as_mut_ptr();

            let mut while_index_pipe=0;
            let mut while_pipe_true=false;
            if pipe_count>0{
                while_pipe_true=true;
            }
            // build the pipe
            while while_pipe_true{
                let ii:isize=while_index_pipe as isize;
                unsafe{
                // println!("offset is {:?}",v_slice.offset(ii) );
                    if pipe(v_slice.offset(ii))<0 {
                        println!("fail to build pipe" );
                    }
                    else{
                        // println!("succeed to pipe");
                    }
                }    
                while_index_pipe+=2;
                if while_index_pipe>=pipe_count*2{
                    break;
                }
            }

            let mut pipe_index=pipe_count; 
            let mut local_command_start=0;  //input's index
            let mut local_command_end=0;  //input's index
            let mut j=0; //index to v's array
            while true{ 
                if pipe_index<=0{
                    break;
                }
                let l1=input_ele.len(); 
                for i in local_command_start..l1{ //[start,l1)
                    if input_ele[i]=="|"{ 
                        break;
                    }
                    local_command_end+=1; //so including this index
                }
                // println!("start command is  {:?}",input_ele[local_command_start] );
                // println!("{:?}",input_ele[local_command_end] );

                // println!("start is {:?}, end is {}", local_command_start,local_command_end);
                // got the start and end, start to fork child process
                unsafe{
                    let fork_result=fork();
                    if fork_result==0{
                        // not the last command, dup2 1
                        if pipe_index-1 >0 {
                            if dup2(v[j+1],1)<0{
                                println!("fail to dup2");
                            }
                        }
                        // not the first command
                        if j!=0{
                            if dup2(v[j-2],0)<0{
                                println!("fail to dup2");
                            }

                        }
                        // start to close
                        for i in &v{
                            close(*i);
                        }
            
                        // execute
                        if input_ele[local_command_start]=="pwd"{
                            exec_pwd();
                            unsafe{     libc::exit(0);      }
                        }
                        else if input_ele[local_command_start]=="cd"{
                            if input_ele.len()==2 {
                                exec_cd(&input_ele[local_command_start+1]);
                            }
                            else{
                                println!("Wrong parameters");
                            }
                            unsafe{     libc::exit(0);      }
                        }
                        else if input_ele[local_command_start]=="exit"{
                            unsafe{     libc::exit(0);      }
                            // break;
                        }
                        else if input_ele[local_command_start]=="history"{
                            // redirect

                            print_history(& history_store);
                            unsafe{     libc::exit(0);      }
                        }
                        else if input_ele[local_command_start]=="kill"{
                            // SIGTERM  15
                            unsafe{
                                kill(input_ele[local_command_start+1].parse().unwrap(),15);   
                            }
                            unsafe{     libc::exit(0);      }
                        }
                        else if input_ele[local_command_start]=="jobs"{
                            // check alive job

                            // println!("{:?}", job_store.len());
                            // println!("{:?}",job_store[0] );
                            if job_store.len()!=0{
                                // let mut fork_result:i32=0;
                                let length_job=job_store.len();
                                let mut index=0;
                                while true{
                                    let fork_result=job_store[index].parse().unwrap();
                                    // wnohang
                                    let mut my_num: i32 = 10;
                                    let status_job:*mut i32 =&mut my_num;
                                    unsafe{
                                        let ret_val=waitpid(fork_result,status_job,1);
                                        if ret_val!=-1{
                                            if ret_val!=0{
                                                job_store.remove(index);
                                                job_store.remove(index);    
                                            }
                                        }
                                        else{
                                            index=index+2;
                                        }
                                    }

                                    if index<length_job-1{
                                        break;
                                    }
                                }
                            }
                            let mut index=0; //only print 2rd, fourth index
                            for i in &job_store{
                                if index %2 ==1{
                                    println!("{}",i);
                                }
                                index=index+1;

                            }
                            unsafe{     libc::exit(0);      }
                        }
                        else{
                            // external command
                            // just execute
                            let command=input_ele[local_command_start].clone();
                            // check <
                            // println!("command is {:?}",command );
                            // println!("external end is {:?},start is {}",local_command_end,local_command_start );
                            if local_command_end-local_command_start>0{
                                let mut left_index=local_command_start;
                                // find the <
                                while true{
                                    if left_index>=local_command_end{
                                        break;
                                    }
                                    if input_ele[left_index]=="<"{
                                        break;
                                    }
                                    left_index+=1;
                                }

                                // redirect
                                // open the file
                                // if no <, left_index will exceed input_ele.len-1
                                if left_index<local_command_end-1{
                                    let file_name=CString::new(input_ele[left_index+1].clone()).unwrap();
                                    let ret_open=open(file_name.as_ptr(),O_RDONLY);
                                    if ret_open<0{
                                        println!("fail to open the file");
                                    }
                                    else{
                                        // open success
                                        if dup2(ret_open,0)<0{
                                            println!("fail to dup2");
                                        }
                                        else{
                                            // dup success
                                            input_ele.remove(left_index);
                                            input_ele.remove(left_index);
                                            
                                        }
                                        close(ret_open);
                                    }
                                }                                
                            }
                            // println!("{:?}",input_ele.len() );
                            // if contains >
                            if local_command_end-local_command_start>0{
                                let mut right_index=local_command_start;
                                // find the <
                                while true{
                                    if right_index>=local_command_end{
                                        break;
                                    }
                                    if input_ele[right_index]==">"{
                                        break;
                                    }
                                    right_index+=1;
                                }

                                // redirect
                                // open the file
                                if right_index<local_command_end-1{
                                    let file_name=CString::new(input_ele[right_index+1].clone()).unwrap();
                                    let ret_write=open(file_name.as_ptr(),O_WRONLY|O_CREAT|O_TRUNC,S_IRWXU);
                                    if ret_write<0{
                                        println!("fail to open the written file");
                                    }
                                    else{
                                        if dup2(ret_write,1)<0{
                                            println!("fail to dup2");
                                        }
                                        else{

                                            input_ele.remove(right_index);
                                            input_ele.remove(right_index);
                                        }
                                        close(ret_write);
                                    }    
                                }                                
                            }
                            let length=input_ele.len();
                            if local_command_end-local_command_start>0{
                                if input_ele[local_command_end-1]=="&"{
                                    // so will this change parents' input_ele
                                    input_ele.remove(local_command_end-1);

                                }
                            }                                        
                            let c_command=CString::new(command).unwrap();
                            
                            let mut temp_vec_command: Vec<String> =Vec::new();
                            for i in local_command_start..local_command_end{
                                temp_vec_command.push(input_ele[i].clone())
                            }
                            let cstr_argv: Vec<_> = temp_vec_command.iter()
                                    .map(|arg| CString::new(arg.as_str()).unwrap())
                                    .collect();
                            let mut p_argv: Vec<_> = cstr_argv.iter()
                                    .map(|arg| arg.as_ptr())
                                    .collect();
                            p_argv.push(std::ptr::null());
                            let p: *const *const c_char = p_argv.as_ptr();
                            execvp(c_command.as_ptr(),p);
                        }
                        // history_store.push(input.clone());
                        // println!("length is {}",job_store.len() );
                    }
                    else if fork_result>0{
                        // in parents' process
                        if input_ele[input_ele.len()-1]=="&"{
                            // record each processes' pid 
                            // input_ele.remove(length-1);
                            let mut temp_vec_command: Vec<String> =Vec::new();
                            for i in local_command_start..local_command_end{
                                temp_vec_command.push(input_ele[i].clone())
                            }
                            let input_string:String=temp_vec_command.join(" ");
                            let fork_result_str=String::from(fork_result.to_string());
                            job_store.push(fork_result_str);
                            job_store.push(input_string);
                        }    
                    }
                    else if fork_result<0{
                        println!("fail to fork in pipe");
                    }
                }
                j=j+2;
                pipe_index=pipe_index-1;
                local_command_start=local_command_end+1;
                local_command_end=local_command_start;  // they should on the same step

                // println!("other end is {:?}",local_command_end );
                // println!("other start is {:?}",local_command_start );
            }

            // in parent's process
            // close the pipe
            unsafe{
                for i in 0..pipe_count{
                    close(v[i]);
                }                

                // wait or not wait
                if input_ele[input_ele.len()-1]!="&"{
                    for i in 0..pipe_count{
                        let mut my_num: i32 = 10;
                        let status: *mut i32 = &mut my_num;
                        wait(status);
                    }
                }    
            }

            // store to history
            history_store.push(input.clone());

        }
        else{

        // no pipe
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
                    // process::exit(0);
                    // println!("calling exit ");
                    unsafe{
                        libc::exit(0);
                    }
                }
                else if input_ele[0]=="history"{
                    print_history(& history_store);
                }
                else if input_ele[0]=="kill"{
                    // SIGTERM  15
                    unsafe{
                        kill(input_ele[1].parse().unwrap(),15);   
                    }
                }
                else if input_ele[0]=="jobs"{
                    // check alive job

                    // println!("{:?}", job_store.len());
                    // println!("{:?}",job_store[0] );
                    if job_store.len()!=0{
                        // let mut fork_result:i32=0;
                        let length_job=job_store.len();
                        let mut index=0;
                        while true{
                            let fork_result=job_store[index].parse().unwrap();
                            // wnohang
                            let mut my_num: i32 = 10;
                            let status_job:*mut i32 =&mut my_num;
                            unsafe{
                                let ret_val=waitpid(fork_result,status_job,1);
                                if ret_val!=-1{
                                    if ret_val!=0{
                                        job_store.remove(index);
                                        job_store.remove(index);    
                                    }
                                }
                                else{
                                    index=index+2;
                                }
                            }

                            if index<length_job-1{
                                break;
                            }
                        }
                    }
                    let mut index=0; //only print 2rd, fourth index
                    for i in &job_store{
                        if index %2 ==1{
                            println!("{}",i);
                        }
                        index=index+1;

                    }
                }
                else{
                    // external command
                    // fork and execvp
                    // fork_result means pid
                    unsafe{
                        let fork_result=fork();

                        if fork_result< 0 {
                            println!("Fail to fork");

                        }
                        else if fork_result==0{
                            // in child's process
                            let command=input_ele[0].clone();
                            // input_ele.remove(0);
                            if input_ele.len()>0{
                                let mut left_index=0;
                                // find the <
                                for i in &input_ele{
                                    if i=="<"{
                                        break;
                                    }
                                    left_index+=1;
                                }
                                // redirect
                                // open the file
                                // if no <, left_index will exceed input_ele.len-1
                                if left_index<input_ele.len()-1{
                                    let file_name=CString::new(input_ele[left_index+1].clone()).unwrap();
                                    let ret_open=open(file_name.as_ptr(),O_RDONLY);
                                    if ret_open<0{
                                        println!("fail to open the file");
                                    }
                                    else{
                                        // open success
                                        if dup2(ret_open,0)<0{
                                            println!("fail to dup2");
                                        }
                                        else{
                                            // dup success
                                            input_ele.remove(left_index);
                                            input_ele.remove(left_index);
                                            
                                        }
                                        close(ret_open);
                                    }
                                }                                
                            }
                            // println!("{:?}",input_ele.len() );
                            // if contains >
                            if input_ele.len()>0{
                                let mut right_index=0;
                                // find the <
                                for i in &input_ele{
                                    if i==">"{
                                        break;
                                    }
                                    right_index+=1;
                                }
                                // redirect
                                // open the file
                                if right_index<input_ele.len()-1{
                                    let file_name=CString::new(input_ele[right_index+1].clone()).unwrap();
                                    let ret_write=open(file_name.as_ptr(),O_WRONLY|O_CREAT|O_TRUNC,S_IRWXU);
                                    if ret_write<0{
                                        println!("fail to open the written file");
                                    }
                                    else{
                                        if dup2(ret_write,1)<0{
                                            println!("fail to dup2");
                                        }
                                        else{

                                            input_ele.remove(right_index);
                                            input_ele.remove(right_index);
                                        }
                                        close(ret_write);
                                    }    
                                }                                
                            }



                            let length=input_ele.len();
                            if length>0{
                                if input_ele[length-1]=="&"{
                                    // so will this change parents' input_ele
                                    input_ele.remove(length-1);

                                }
                            }    
                            
                            let c_command=CString::new(command).unwrap();
                            // input_ele.iter().next();
                            // input_ele.iter().next() ;
                            // println!("------left is {:?}",input_ele.iter().next() );

                            // fetch the correct data?
                            let cstr_argv: Vec<_> = input_ele.iter()
                                    .map(|arg| CString::new(arg.as_str()).unwrap())
                                    .collect();
                            let mut p_argv: Vec<_> = cstr_argv.iter()
                                    .map(|arg| arg.as_ptr())
                                    .collect();
                            // for i in &p_argv{
                            //     println!("====={:?}", i);
                            // }

                            p_argv.push(std::ptr::null());
                            let p: *const *const c_char = p_argv.as_ptr();
                            execvp(c_command.as_ptr(),p);
                            // println!("ret_val is {}",ret_val);
                            
                        }
                        else {
                            // in parents' process
                            
                            // if execute in the background,no need to wait
                            let length=input_ele.len();
                            // println!("{:?}",input_ele );
                            if input_ele[length-1]=="&"{
                                // remove &
                                input_ele.remove(length-1);
                                let input_string:String=input_ele.join(" ");
                                let fork_result_str=String::from(fork_result.to_string());
                                job_store.push(fork_result_str);
                                job_store.push(input_string);

                                // for i in input_ele_job{
                                //     input_string.push_str(i);

                                // }
                                // store the command
                                // println!("{:?}",input_string );
                                // add_to_job(&input_ele,);    
                            }
                            else{
                                // else do it interactively, so wait
                                let mut my_num: i32 = 10;
                                let status: *mut i32 = &mut my_num;
                                // wnohang
                                waitpid(fork_result,status,0);

                            }    
                        }
                    }
                }
            }
            history_store.push(input.clone());
                // println!("length is {}",job_store.len() );
             
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
}



fn main() {
    // println!("below is the output");
    // println!("{number:>width$}", number=1, width=6);

    start_dash();
}

