// Computes RAM[2] =
// max(RAM[0],RAM[1])
@R0
D=M
@R1
D=D-M
@OUTPUT_RAM0
D;JGT
// Output RAM[1]
@R1
D=M
@R2
M=D
@END
0;JMP
(OUTPUT_RAM0)
@R0
D=M
@R2
M=D
(END)
@END
0;JMP
