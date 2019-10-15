#![cfg(test)]

use crate::db::ChalkDatabase;
use crate::query::LoweringDatabase;
use chalk_ir;
use chalk_solve::ext::*;
use chalk_solve::{Solution, SolverChoice};

#[cfg(feature = "bench")]
mod bench;
mod coherence;
mod slg;
mod wf_lowering;

fn assert_result(result: &Option<Solution>, expected: &str) {
    let result = match result {
        Some(v) => format!("{}", v),
        None => format!("No possible solution"),
    };

    println!("expected:\n{}", expected);
    println!("actual:\n{}", result);

    let expected1: String = expected.chars().filter(|w| !w.is_whitespace()).collect();
    let result1: String = result.chars().filter(|w| !w.is_whitespace()).collect();
    assert!(!expected1.is_empty() && result1.starts_with(&expected1));
}

macro_rules! test {
    (program $program:tt $($goals:tt)*) => {
        test!(@program[$program]
              @parsed_goals[]
              @unparsed_goals[$($goals)*])
    };

    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[]) => {
        solve_goal(stringify!($program), vec![$($parsed_goals),*])
    };

    // goal { G } yields { "Y" } -- test both solvers behave the same (the default)
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields { $expected:expr }
        $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), SolverChoice::default(), $expected)
              ]
              @unparsed_goals[$($unparsed_goals)*])
    };

    // goal { G } yields[C1] { "Y1" } yields[C2] { "Y2" } -- test that solver C1 yields Y1
    // and C2 yields Y2
    //
    // Annoyingly, to avoid getting a parsing ambiguity error, we have
    // to distinguish the case where there are other goals to come
    // (this rule) for the last goal in the list (next rule). There
    // might be a more elegant fix than copy-and-paste but this works.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt $(yields[$($C:expr),+] { $expected:expr })*
            goal $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[$($parsed_goals)*
                            $($((stringify!($goal), $C, $expected))+)+]
              @unparsed_goals[goal $($unparsed_goals)*])
    };

    // same as above, but for the final goal in the list.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt $(yields[$($C:expr),+] { $expected:expr })*
    ]) => {
        test!(@program[$program]
              @parsed_goals[$($parsed_goals)*
                            $($((stringify!($goal), $C, $expected))+)+]
              @unparsed_goals[])
    };
}

fn solve_goal(program_text: &str, goals: Vec<(&str, SolverChoice, &str)>) {
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));

    let mut db = ChalkDatabase::with(
        &program_text[1..program_text.len() - 1],
        SolverChoice::default(),
    );

    for (goal_text, solver_choice, expected) in goals {
        if db.solver_choice() != solver_choice {
            db.set_solver_choice(solver_choice);
        }

        let program = db.checked_program().unwrap();

        chalk_ir::tls::set_current_program(&program, || {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = db
                .parse_and_lower_goal(&goal_text[1..goal_text.len() - 1])
                .unwrap();

            println!("using solver: {:?}", solver_choice);
            let peeled_goal = goal.into_peeled_goal();
            let result = db.solve(&peeled_goal);
            assert_result(&result, expected);
        });
    }
}

mod auto_traits;
mod coherence_goals;
mod coinduction;
mod cycle;
mod dyn_trait;
mod implied_bounds;
mod impls;
mod negation;
mod projection;
mod unify;
mod wf_goals;

#[test]
fn inscope() {
    test! {
        program {
            trait Foo { }
        }

        goal {
            InScope(Foo)
        } yields {
            "No possible solution"
        }

        goal {
            if (InScope(Foo)) {
                InScope(Foo)
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
