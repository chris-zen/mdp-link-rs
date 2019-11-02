target remote :3333

# print demangled symbols
set print asm-demangle on

monitor arm semihosting enable

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind
break main

load

# start the process but immediately halt the processor
stepi
