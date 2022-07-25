# TODO

Splitting and merging (or any future insert or delete mutation) screws up all
addresses on the stack of a processor that happens to be beyond the split or
merge point. To make things more resilient we could switch to a number of
read/write heads that are not on the stack and can be adjusted when a split or
mutation occurs. Heads are essentially variables.

Head management:

- nr HEAD - switch current head. Head may not be initialized, if so it's insert
- ADDR - set current head to address of instruction
- source COPY - copy source head address into current head
- READ - read at current head
- value WRITE - write value to current head
- amount BACK - move current head back by amount
- amount FORWARD - move current head forward by amount

JMP, CALL is defined at going to the current head

Less fancy stack management is required, and processors can survive movement.
