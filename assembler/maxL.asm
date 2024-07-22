// Computes RAM[2] =
// max(RAM[0],RAM[1])
@0
D=M
@1
D=D-M
@12
D;JGT
// Output RAM[1]
@1
D=M
@2
M=D
@16
0;JMP
@0
D=M
@2
M=D
@16
0;JMP
