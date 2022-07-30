pub const PROGRAM_TEXT: &str = "
# This is a full-on replicator that can be used
# to see the simulation. It replicates itself, 
# tries to grow memory, splits into two

ADDR  # h0 = start
N1
HEAD
N0
COPY  # h1 = reader
N2
HEAD
N0    
COPY  # h2 = writer
N8
N8
MUL
DUP
FORWARD # h2 forward 64
DUP
ADD # 128 on stack
N3
HEAD
N2
COPY  # h3 = h2, start offspring
N4
HEAD  
ADDR  # h4 = loop
EAT
GROW
N1
HEAD
READ  # read from h1
N1
FORWARD
N2
HEAD
WRITE # write to h2
N1
FORWARD
DUP   # duplicate 128 
N3
DISTANCE # distance h2 and h3
SWAP
LT    # if distance < 128
N4
HEAD
JMPIF # jump to h4, loop
N3
HEAD
START # start offspring at h3
N2
BACKWARD
RND   # random direction
SPLIT # split 2 positions earlier
N0
HEAD
JMP   # jump to first addr";
