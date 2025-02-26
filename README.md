# qASM

Not to be confused with OpenQASM. An assembly language that contains both quantum and classical instructions.
This repository contains the emulator for qASM.

# Usage

Currently there are no prebuilt binaries so you will have to use cargo manually to build and run the emulator.
The general layout of qASM code is:

```
qbits n
cbits n
qregs n
cregs n

<code>

hlt
```

The order of the headers is important and should not be changed. The `qbits` header specifies the number of
qubits in each quantum register. The `cbits` header specifies the number of classical bits in each classical
register. The `qregs` header specifies the number of quantum registers. The `cregs` header specifies the number
of classical registers. The `n` after each header is any non-negative number. Operands/arguments for each
instructions are delimited by spaces and not commas.

## Quantum Instructions
The general format for quantum instructions are:
```
op qn <other arguments>
```
Where `op` is the name/opcode, `qn` specifies a specific qubit `n` of the currently selected quantum register.
`<other arguments>` can include more qubits as arguments, or in the case of some instructions, a rotation expressed
as a rational multiple of pi, in the format `[n]pi[/n]`, where `n` can be any non-negative number, and items
in `[]` are optional. Quantum registers can be selected via the `qsel` instruction, which has the general format
`qsel qrn` where `n` is any non-negative number.

List of currently implemented quantum instructions:

| Quantum Gate        | Instruction name | Syntax example      | Explanation |
| ------------------- | ---------------- | ------------------- | ----------- |
| Hadamard            | h                | `h q0`              | Applies a Hadamard to qubit 0 |
| CNOT                | cnot             | `cnot q0 q1`        | Applies a CNOT to qubit 1 with qubit 0 being the control |
| CCNOT/Toffoli       | ccnot            | `ccnot q0 q1 q2`    | Applies a Toffoli to qubit 2 with qubit 0 and qubit 1 being the controls |
| Pauli X             | x                | `x q0`              | Applies a Pauli X to qubit 0 |
| Pauli Y             | y                | `y q0`              | Applies a Pauli Y to qubit 0 |
| Pauli Z             | z                | `z q0`              | Applies a Pauli Z to qubit 0 |
| Rx                  | rx               | `rx q0 pi/3`        | Rotates the statevector of qubit 0 by pi/3 radians along X axis on bloch sphere |
| Ry                  | ry               | `ry q0 pi`          | Rotates the statevector of qubit 0 by pi radians along Y axis on bloch sphere |
| Rz                  | rz               | `rz q0 pi/4`        | Rotates the statevector of qubit 0 by pi/4 radians along Z axis on bloch sphere |
| U gate              | u                | `u q0 pi pi/3 pi/6` | Rotates the statevector of qubit 0 by the 3 Euler angles pi, pi/3, pi/6 |
| S gate              | s                | `s q0`              | Applies an S gate to qubit 0 |
| T gate              | t                | `t q0`              | Applies a T gate to qubit 0 |
| S-dagger            | sdg              | `sdg q0`            | Applies a S-dagger or the inverse of S gate to qubit 0 |
| T-dagger            | tdg              | `tdg q0`            | Applies a T-dagger or the inverse of T gate to qubit 0 |
| Phase gate          | p                | `p q0 pi/3`         | Applies a rotation to the $\ket{1}$ state by pi/3 radians |
| Controlled Hadamard | ch               | `ch q0 q1`          | Applies a controlled Hadamard to qubit 1 with qubit 0 being the control |
| Controlled Pauli Y  | cy               | `cy q0 q1`          | Applies a controlled Pauli Y to qubit 1 with qubit 0 being the control |
| Controlled Pauli Z  | cz               | `cz q0 q1`          | Applies a controlled Pauli Z to qubit 1 with qubit 0 being the control |
| Controlled Phase    | cp               | `cp q0 q1 pi/2`     | Applies a controlled Phase gate to qubit 1 of pi/2 radians with qubit 0 being the control |
| Swap                | swap             | `swap q0 q1`        | Swaps the state of qubits 0 and 1 |
| Square Root NOT     | sqrtx            | `sqrtx q0 `         | Applies a sqrt(NOT)/sqrt(Pauli X) to qubit 0 |
| Square Root Swap    | sqrtswp          | `sqrtswp q0 q1`     | Applies a sqrt(Swap) to qubits 0 and 1, halfway swapping their state |
| Controlled Swap     | cswap            | `cswap q0 q1 q2`    | Swaps the state of qubits 1 and 2 with qubit 0 being the control |
| Measure             | m                | `m q0 cr1 c3`       | Measures the state of qubit 0 into 3rd bit of classical register 1 |

## Classical Instructions
General format for classical instructions are:
```
op <operands>
```
Where `op` is the name/opcode, operands may include `crn`, which specifies a specific classical register `n`, or
an immediate literal value (for now non-negative due to not implemented in parser yet) Other than these differences,
they behave basically the same as any other assembly language instructions.

List of currently implemented classical instructions:

*Note: The value of a register refers to the value stored in the register. The value of an immediate is the immediate number itself.*

*Note 2: An operand can either be a register or immediate unless a restriction is specified.*

| Instruction name | Description |
| ---------------- | ----------- |
| add              | op1 = op2 + op3. op1 is always a register. |
| sub              | op1 = op2 - op3. op1 is always a register. |
| mult             | op1 = op2 * op3. op1 is always a register. All values are treated unsigned. |
| umult            | op1 = (op2 * op3) >> (cbits/2). op1 is always a register. All values are treated unsigned. |
| div              | op1 = op2 / op3. op1 is always a register. Performs integer division. All values are treated unsigned. |
| smult            | op1 = op2 * op3. op1 is always a register. All values are treated signed. |
| sumult           | op1 = (op2 * op3) >> (cbits/2). op1 is always a register. All values are treated signed. |
| sdiv             | op1 = op2 / op3. op1 is always a register. Performs integer division. All values are treated signed. |
| not              | op1 = ~op2. op1 is always a register. |
| and              | op1 = op2 & op3. op1 is always a register. |
| or               | op1 = op2 \| op3. op1 is always a register. |
| xor              | op1 = op2 ^ op3. op1 is always a register. |
| nand             | op1 = ~(op2 & op3). op1 is always a register. |
| nor              | op1 = ~(op2 \| op3). op1 is always a register. |
| xnor             | op1 = ~(op2 ^ op3). op1 is always a register. |

## Misc. Instructions
These instructions are here because I wanted them separate from [Classical Instructions](#classical-instructions)

| Instruction name | Description |
| ---------------- | ----------- |
| cmp              | Updates flags based on comparing values in op1 and op2. op1 is always a register. |
| jmp              | Unconditionally jump to a label. |
| jeq              | Jump to label if comparison resulted in EQ flag set. |
| jne              | Jump to label if comparsion did not result in EQ flag set. |
| jg               | Jump to label if comparison resulted in GREATER flag set. |
| jge              | Jump to label if comparison resulted in GREATER or RQ flag set. |
| jl               | Jump to label if comparison resulted in LESSER flag set. |
| jle              | Jump to label if comparison resulted in LESSER or EQ flag set. |
| hlt              | Halt the program. |

# Examples
This program simulates the $\ket{\Phi^+}$ bell state:
```
qbits 2
cbits 2
qregs 1
cregs 1

qsel qr0
h q0
cnot q0 q1

m q0 cr0 c0
m q1 cr0 c1

hlt
```
