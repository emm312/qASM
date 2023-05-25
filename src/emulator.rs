use std::f64::consts::{
    FRAC_1_SQRT_2 as ONE_BY_ROOT_2, FRAC_PI_2 as PI_BY_2, FRAC_PI_4 as PI_BY_4,
};
use std::fmt::Display;
use std::num::Wrapping;

use lazy_static::lazy_static;

use num_traits::{One, Zero};

use nalgebra::Complex;
use nalgebra::{dmatrix, DMatrix};
use nalgebra::{DVector, Vector2, Vector4};

use crate::ast::*;

type Complexf64 = Complex<f64>;
type DCf64Mat = DMatrix<Complexf64>;

lazy_static! {
    static ref MAT_2_IDENTITY: DCf64Mat = dmatrix![
        Complex::one(), Complex::zero();
        Complex::zero(), Complex::one();
    ];
    static ref HADAMARD_MAT: DCf64Mat = dmatrix![
        Complex::from(ONE_BY_ROOT_2), Complex::from(ONE_BY_ROOT_2);
        Complex::from(ONE_BY_ROOT_2), Complex::from(-ONE_BY_ROOT_2);
    ];
    static ref PAULI_X_MAT: DCf64Mat = dmatrix![
        Complex::zero(), Complex::one();
        Complex::one(), Complex::zero();
    ];
    static ref PAULI_Y_MAT: DCf64Mat = dmatrix![
        Complex::zero(), -Complex::i();
        Complex::i(), Complex::zero();
    ];
    static ref PAULI_Z_MAT: DCf64Mat = dmatrix![
        Complex::one(), Complex::zero();
        Complex::zero(), -Complex::one();
    ];
    static ref S_MAT: DCf64Mat = r1(PI_BY_2);
    static ref T_MAT: DCf64Mat = r1(PI_BY_4);
    static ref S_DG_MAT: DCf64Mat = S_MAT.adjoint();
    static ref T_DG_MAT: DCf64Mat = T_MAT.adjoint();
    static ref SQRT_X_MAT: DCf64Mat = dmatrix![
        Complex::new(0.5, 0.5), Complex::new(0.5, -0.5);
        Complex::new(0.5, -0.5), Complex::new(0.5, 0.5);
    ];
    static ref SQRT_SWAP_MAT: DCf64Mat = dmatrix![
        Complex::one(), Complex::zero(), Complex::zero(), Complex::zero();
        Complex::zero(), Complex::new(0.5, 0.5), Complex::new(0.5, -0.5), Complex::zero();
        Complex::zero(), Complex::new(0.5, -0.5), Complex::new(0.5, 0.5), Complex::zero();
        Complex::zero(), Complex::zero(), Complex::zero(), Complex::one();
    ];
}

#[inline]
fn u(theta: f64, phi: f64, lambda: f64) -> DCf64Mat {
    let half_theta = theta / 2f64;
    dmatrix![
        Complex::from(half_theta.cos()), -half_theta.sin() * Complex::new(lambda.cos(), lambda.sin());
        half_theta.sin() * Complex::new(phi.cos(), phi.sin()), half_theta.cos() * Complex::new((lambda + phi).cos(), (lambda + phi).sin());
    ]
}
#[inline]
fn rx(theta: f64) -> DCf64Mat {
    u(theta, -PI_BY_2, PI_BY_2)
}
#[inline]
fn ry(theta: f64) -> DCf64Mat {
    u(theta, 0f64, 0f64)
}
#[inline]
fn rz(theta: f64) -> DCf64Mat {
    let half_theta = theta / 2f64;
    // If only i could do this
    // Complex::new(half_theta.cos(), -half_theta.sin()) * r1(theta)
    dmatrix![
        Complex::new(half_theta.cos(), -half_theta.sin()), Complex::zero();
        Complex::zero(), Complex::new(half_theta.cos(), half_theta.sin());
    ]
}
#[inline]
fn r1(theta: f64) -> DCf64Mat {
    u(0f64, theta, 0f64)
}

pub struct Emulator<'a> {
    prog: &'a Program,

    qbits: u16,
    cbits: u16,

    qregs: Vec<DVector<Complexf64>>,
    cregs: Vec<Word>,

    pc: usize,
    qreg_sel: usize,
}

impl<'a> Emulator<'a> {
    pub fn new(prog: &'a Program) -> Self {
        let qbits = prog.headers.0;
        let cbits = prog.headers.1;
        let qregs = prog.headers.2;
        let cregs = prog.headers.3;

        Self {
            prog,
            qbits,
            cbits,

            qregs: vec![
                {
                    let mut dvec = DVector::zeros(1 << qbits);
                    dvec[0] = Complex::one();
                    dvec
                };
                qregs
            ],
            cregs: vec![
                Word {
                    bits: cbits as u16,
                    data: Wrapping(0),
                };
                cregs
            ],

            pc: 0,
            qreg_sel: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), EmulatorError> {
        loop {
            let res = self.step()?;
            if res == StepResult::Halted {
                break;
            }

            self.pc += 1
        }
        Ok(())
    }

    fn step(&mut self) -> Result<StepResult, EmulatorError> {
        let Some(&inst) = self.prog.instructions.get(self.pc) else {
            return Err(EmulatorError::PCOutOfBounds)
        };

        match inst {
            // Quantum Instructions
            Inst::Qsel(qreg) => {
                if qreg > self.qregs.len() {
                    return Err(EmulatorError::QregIndexOutOfBounds);
                }
                self.qreg_sel = qreg
            }
            Inst::Id(qbit) => self.apply_mat_1(&MAT_2_IDENTITY, qbit),
            Inst::Hadamard(qbit) => self.apply_mat_1(&HADAMARD_MAT, qbit),
            Inst::Cnot(qbit1, qbit2) => {
                let qreg = &mut self.qregs[self.qreg_sel];

                let qbit1_mask = 1 << qbit1;
                let qbit2_mask = 1 << qbit2;

                for state in 0..(1 << self.qbits) {
                    if state & qbit1_mask == 0 || state & qbit2_mask != 0 {
                        continue;
                    }
                    qreg.swap_rows(state, state | qbit2_mask)
                }
            }
            Inst::Ccnot(qbit1, qbit2, qbit3) => {
                let qreg = &mut self.qregs[self.qreg_sel];

                let qbit1_mask = 1 << qbit1;
                let qbit2_mask = 1 << qbit2;
                let qbit3_mask = 1 << qbit3;

                for state in 0..(1 << self.qbits) {
                    if state & qbit1_mask == 0 || state & qbit2_mask == 0 || state & qbit3_mask != 0
                    {
                        continue;
                    }
                    qreg.swap_rows(state, state | qbit3_mask)
                }
            }
            Inst::X(qbit) => self.apply_mat_1(&PAULI_X_MAT, qbit),
            Inst::Y(qbit) => self.apply_mat_1(&PAULI_Y_MAT, qbit),
            Inst::Z(qbit) => self.apply_mat_1(&PAULI_Z_MAT, qbit),
            Inst::Rx(qbit, rot) => self.apply_mat_1(&rx(rot.get_angle()), qbit),
            Inst::Ry(qbit, rot) => self.apply_mat_1(&ry(rot.get_angle()), qbit),
            Inst::Rz(qbit, rot) => self.apply_mat_1(&rz(rot.get_angle()), qbit),
            Inst::U(qbit, theta, phi, lambda) => self.apply_mat_1(
                &u(theta.get_angle(), phi.get_angle(), lambda.get_angle()),
                qbit,
            ),
            Inst::S(qbit) => self.apply_mat_1(&S_MAT, qbit),
            Inst::T(qbit) => self.apply_mat_1(&T_MAT, qbit),
            Inst::Sdg(qbit) => self.apply_mat_1(&S_DG_MAT, qbit),
            Inst::Tdg(qbit) => self.apply_mat_1(&T_DG_MAT, qbit),
            Inst::P(qbit, rot) => self.apply_mat_1(&r1(rot.get_angle()), qbit),
            Inst::Ch(qbit1, qbit2) => self.apply_controlled_mat(&HADAMARD_MAT, vec![qbit1], qbit2),
            Inst::Cy(qbit1, qbit2) => self.apply_controlled_mat(&PAULI_Y_MAT, vec![qbit1], qbit2),
            Inst::Cz(qbit1, qbit2) => self.apply_controlled_mat(&PAULI_Z_MAT, vec![qbit1], qbit2),
            Inst::Swap(qbit1, qbit2) => {
                let qreg = &mut self.qregs[self.qreg_sel];

                let qbit1_mask = 1 << qbit1;
                let qbit2_mask = 1 << qbit2;

                for state in 0..(1 << self.qbits) {
                    if state & qbit1_mask != 0 || state & qbit2_mask != 0 {
                        continue;
                    }
                    qreg.swap_rows(state | qbit1_mask, state | qbit2_mask)
                }
            }
            Inst::SqrtX(qbit) => self.apply_mat_1(&SQRT_X_MAT, qbit),
            Inst::SqrtSwap(qbit1, qbit2) => self.apply_mat_2(&SQRT_SWAP_MAT, qbit1, qbit2),

            // Classical Instructions
            Inst::Add(r1, r2, r3) => {
                let val1 = self.get_val(r2);
                let val2 = self.get_val(r3);
                self.cregs[r1].set_to(val1 + val2);
            }

            Inst::Hlt => return Ok(StepResult::Halted),
        }

        Ok(StepResult::Continue)
    }

    fn get_val(&self, thing: Operand) -> Wrapping<i64> {
        match thing {
            Operand::Imm(imm) => Wrapping(imm),
            Operand::Reg(reg) => self.cregs[reg].get_val(),
        }
    }

    fn apply_mat_1(&mut self, mat: &DCf64Mat, qbit: usize) {
        let qreg = &mut self.qregs[self.qreg_sel];

        let mut in_gate_state = Vector2::zeros();
        let mut out_gate_state = Vector2::zeros();

        let qbit_mask = 1 << qbit;

        for state in 0..(1 << self.qbits) {
            if state & qbit_mask != 0 {
                continue;
            }

            in_gate_state[0] = qreg[state];
            in_gate_state[1] = qreg[state | qbit_mask];

            mat.mul_to(&in_gate_state, &mut out_gate_state);

            qreg[state] = out_gate_state[0];
            qreg[state | qbit_mask] = out_gate_state[1];
        }
    }

    fn apply_mat_2(&mut self, mat: &DCf64Mat, qbit1: usize, qbit2: usize) {
        let qreg = &mut self.qregs[self.qreg_sel];

        let mut in_gate_state = Vector4::zeros();
        let mut out_gate_state = Vector4::zeros();

        let qbit1_mask = 1 << qbit1;
        let qbit2_mask = 1 << qbit2;

        for state in 0..(1 << self.qbits) {
            if state & qbit1_mask != 0 || state & qbit2_mask != 0 {
                continue;
            }

            in_gate_state[0] = qreg[state];
            in_gate_state[1] = qreg[state | qbit1_mask];
            in_gate_state[2] = qreg[state | qbit2_mask];
            in_gate_state[3] = qreg[state | qbit1_mask | qbit2_mask];

            mat.mul_to(&in_gate_state, &mut out_gate_state);

            qreg[state] = out_gate_state[0];
            qreg[state | qbit1_mask] = out_gate_state[1];
            qreg[state | qbit2_mask] = out_gate_state[2];
            qreg[state | qbit1_mask | qbit2_mask] = out_gate_state[3];
        }
    }

    fn apply_controlled_mat(&mut self, mat: &DCf64Mat, controls: Vec<usize>, target: usize) {
        let qreg = &mut self.qregs[self.qreg_sel];

        let controls_mask = controls
            .iter()
            .map(|control| 1 << control)
            .collect::<Vec<_>>();
        let target_mask = 1 << target;

        let mut in_gate_state = Vector2::zeros();
        let mut out_gate_state = Vector2::zeros();

        for state in 0..(1 << self.qbits) {
            if controls_mask
                .iter()
                .map(|mask| state & mask == 0)
                .any(|b| b)
                || state & target_mask != 0
            {
                continue;
            }

            in_gate_state[0] = qreg[state];
            in_gate_state[1] = qreg[state | target_mask];

            mat.mul_to(&in_gate_state, &mut out_gate_state);

            qreg[state] = out_gate_state[0];
            qreg[state | target_mask] = out_gate_state[1];
        }
    }
}

impl<'a> Display for Emulator<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Emulator state: (qbits={}, cbits={})",
            self.qbits, self.cbits
        )?;
        writeln!(f, "qregs=[")?;
        for qreg in &self.qregs {
            write!(f, "{}", qreg)?
        }
        writeln!(f, "],")?;

        write!(f, "cregs={:?}", self.cregs)?;

        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Word {
    pub bits: u16,
    pub data: Wrapping<i64>,
}

impl Word {
    pub fn get_val(&self) -> Wrapping<i64> {
        self.data & Wrapping(((1 << self.bits) as i64).wrapping_sub(1))
    }

    pub fn set_to(&mut self, val: Wrapping<i64>) {
        self.data = val & Wrapping(((1 << self.bits) as i64).wrapping_sub(1))
    }
}

impl std::fmt::Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)?;
        Ok(())
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_val())?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StepResult {
    Halted,
    Continue,
}

#[derive(Debug, Clone, Copy)]
pub enum EmulatorError {
    PCOutOfBounds,
    QregIndexOutOfBounds,
}
