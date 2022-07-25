# TODO

- I suspect there is a bug in CALL and CALLIF, writ etests.

- Splitting and merging (or any future insert or delete mutation) screws up all
  addresses on the stack of a processor that happens to be beyond the split or
  merge point. To make things more resilient we could switch to a number of
  read/write heads that are not on the stack and can be adjusted when a split
  or mutation occurs.
