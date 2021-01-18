# fileserve-simple

## USAGE:
    fileserve-simple [OPTIONS] [PORT]

## FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

## OPTIONS:
    -b, --bind <ADDRESS>           Alternative bind address [default: 127.0.0.1]
    -d, --directory <DIRECTORY>    Alternative directory to serve [deafault: curent directory
    -w, --workers <AMOUNT>         Number of requests that can be handled concurrently [default: 10]

## ARGS:
    <PORT>    Port to run on [default: 8080]
