# Homework 1: Rush (Rust Shell)

## Updates

* 2017-03-23: Simplified how to use the `libc` crate.
* 2017-03-09: Updated the `history` built-in command and the reference program.
* 2017-03-07: Updated the `jobs` built-in command and the reference program.

## Introduction

A shell is a command interpreter for the system. On many Linux systems, *bash* is the default interactive shell, and *dash* is the default script interpreter. In this homework, you will implement a subset of *dash* called *rush*. You will learn system calls for process management and inter-process communications (IPC).

You may find my [reference program](rush) (SHA-256: 9676ff69b18b438af104c90cd5c2d20aad6b9f15665b8792d14d87f08a296907) useful.

## Specification

### Input language

The shell reads commands from the standard input and execute them. We specify the input using Extended Backus-Naur Form (EBNF):

```
Production  = production_name "=" [ Expression ] "." .
Expression  = Alternative { "|" Alternative } .
Alternative = Term { Term } .
Term        = production_name | token [ "…" token ] | Group | Option | Repetition .
Group       = "(" Expression ")" .
Option      = "[" Expression "]" .
Repetition  = "{" Expression "}" .
```

Productions are expressions constructed from terms and the following operators, in increasing precedence:

```
|   alternation
()  grouping
[]  option (0 or 1 times)
{}  repetition (0 to n times)
```

The input is encoded in Unicode. Each pair of adjacent tokens in the input are separated by one or more Unicode white space, except that the new line character need not be preceded or followed by white space. The following specification describes the input language.

```
Input = { CommandLine } .
CommandLine = [ Command [ "<" FileName ] { "|" Command } [ ">" FileName ] [ "&" ] ] new_line .
Command = ( BuiltInCommand | ExecutableName ) { Argument } .
```

### Built-in commands

* `cd` *directory* <br>
Sets the current working directory to *directory*.

* `exit` <br>
Exits the shell.

* `history` <br>
Prints all the command lines that the user has entered in the chronological order. For each line
  1. prints a counter that starts from 1, occupies 5 spaces, and is right-aligned;
  1. prints two spaces;
  1. prints the line;
  
  For example, 
```
    1  ls
    2  ls | cat
    3  cat < foo | cat | cat > bar
    4  sleep 10 &
```

* `jobs` <br>
Prints the live command lines in the chronological order. For each command line,
prints its canonical representation as follows:

	* Prints all the tokens: built-in commands, executables, arguments, file names for I/O redirection, `>`, `<`, and `|`. Do *not* print `&`.
	* Separate every pair of adjacent tokens by one white space. Do not add white space at the beginning or end of the line.

	Do *not* print the dead command lines, whose commands have all finished.

* `kill` *pid* <br>
Sends the signal `SIGTERM` to the process *pid*.

* `pwd` <br>
Prints the current working directory.

If a line contains a single built-in command, the command executes in the current process and ignores I/O direction.

### External commands

An external command is the name of an executable file. If the file name contains at least a slash (`/`), executes the file. Otherwise, searches each entry in the `PATH` environment variable in turn for the command. 

External commands execute in child processes.

### I/O redirection

* `<` *filename*

Reads from *filename* instead of `stdio`.

* `>` *filename*

Writes to *filename* instead of `stdout`.

### Pipes

```
command_1 | command_2 | ... | command_n
```

Runs each command in a child process, where the standard output from `command_i` becomes the standard input into `command_{i+1}`. Optionally, the first command may redirect its input from a file, and the last command its output to a file.

Each command may be either external or built-in.

### Background processes

If a line has no trailing `&`, the shell waits for all the commands on this line to finish before reading the next line. Otherwise, the commands on the line run in the "background", and the shell reads the next line immediately.

### Error handling

We will test your shell with only valid input. We encourage, but not require, your program to handle error input.

### Output

The shell prints a prompt `$` followed by a space before reading each line of input.

## Rust hints

The [libc](https://github.com/rust-lang/libc) crate provides native bindings to the types and functions used in system calls. If you use it, put the following in the `[dependencies]` section in your `Cargo.toml`:

```
libc = "0.2"
```

[std::ffi::CStr](https://doc.rust-lang.org/std/ffi/struct.CStr.html) and [std::ffi::CString](https://doc.rust-lang.org/std/ffi/struct.CString.html) represent C strings and provide methods for converting to and from Rust strings.

## Academic integrity

This is an individual homework. While you may share ideas, algorithms, and test cases with others, under no circumstances may you exchange solution code with others. Examples of cheating include (but are not limited to):

* Read or possess solution code written by other people, including people outside this course.
* Submit to the gradebot code written by other people or derived from the code written by other people.
* Allow other people to read or possess your solution code either intentionally or negligently, e.g., by posting your code on a web site or leaving the computer containing your code unattended and unlocked. You are responsible for exercising due diligence in safeguarding your code.

## Submission

Register at the [gradebot](https://gradebot.org/) and follow its instructions to set up your private key. Then,

```
$ cargo new rush --bin
$ cd rush
$ # Save your program in `src/main.rs` and documentation in `README.md`
$ git add Cargo.toml src/main.rs
$ git commit
$ git remote add origin metastasis@gradebot.org:user/{username}/4/1
$ git push origin master
```

## Bonuses

* If your program passes all the test cases by Sunday, March 12, you will get 25% extra credit.

* We will also give extra credits for implementing other features in *dash*. Document these features in `README.md`.

Last updated: 2017-03-23
