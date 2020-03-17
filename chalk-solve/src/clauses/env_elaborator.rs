use super::program_clauses::ToProgramClauses;
use crate::clauses::builder::ClauseBuilder;
use crate::clauses::match_type_name;
use crate::DomainGoal;
use crate::FromEnv;
use crate::ProgramClause;
use crate::RustIrDatabase;
use crate::Ty;
use crate::TyData;
use chalk_ir::interner::Interner;
use chalk_ir::AliasTy;
use rustc_hash::FxHashSet;

/// When proving a `FromEnv` goal, we elaborate all `FromEnv` goals
/// found in the environment.
///
/// For example, when `T: Clone` is in the environment, we can prove
/// `T: Copy` by adding the clauses from `trait Clone`, which includes
/// the rule `FromEnv(T: Copy) :- FromEnv(T: Clone)
pub(super) fn elaborate_env_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    in_clauses: &Vec<ProgramClause<I>>,
    out: &mut FxHashSet<ProgramClause<I>>,
) {
    let mut this_round = vec![];
    let mut visitor = EnvElaborator::new(db, &mut this_round);
    for clause in in_clauses {
        visitor.visit_program_clause(&clause);
    }
    out.extend(this_round);
}

struct EnvElaborator<'me, I: Interner> {
    db: &'me dyn RustIrDatabase<I>,
    builder: ClauseBuilder<'me, I>,
}

impl<'me, I: Interner> EnvElaborator<'me, I> {
    fn new(db: &'me dyn RustIrDatabase<I>, out: &'me mut Vec<ProgramClause<I>>) -> Self {
        EnvElaborator {
            db,
            builder: ClauseBuilder::new(db, out),
        }
    }

    fn visit_alias_ty(&mut self, alias_ty: &AliasTy<I>) {
        debug!("EnvElaborator::visit_alias_ty(alias_ty={:?})", alias_ty);
        self.db
            .associated_ty_data(alias_ty.associated_ty_id)
            .to_program_clauses(&mut self.builder);
    }

    fn visit_ty(&mut self, ty: &Ty<I>) {
        debug!("EnvElaborator::visit_ty(ty={:?})", ty);
        match ty.data() {
            TyData::Apply(application_ty) => {
                match_type_name(&mut self.builder, application_ty.name)
            }
            TyData::Placeholder(_) => {}

            TyData::Alias(alias_ty) => {
                self.visit_alias_ty(alias_ty);
            }

            // FIXME(#203) -- We haven't fully figured out the implied
            // bounds story around `dyn Trait` types.
            TyData::Dyn(_) => (),

            TyData::Function(_) | TyData::BoundVar(_) | TyData::InferenceVar(_) => (),
        }
    }

    fn visit_from_env(&mut self, from_env: &FromEnv<I>) {
        debug!("EnvElaborator::visit_from_env(from_env={:?})", from_env);
        match from_env {
            FromEnv::Trait(trait_ref) => {
                let trait_datum = self.db.trait_datum(trait_ref.trait_id);

                trait_datum.to_program_clauses(&mut self.builder);

                // If we know that `T: Iterator`, then we also know
                // things about `<T as Iterator>::Item`, so push those
                // implied bounds too:
                for &associated_ty_id in &trait_datum.associated_ty_ids {
                    self.db
                        .associated_ty_data(associated_ty_id)
                        .to_program_clauses(&mut self.builder);
                }
            }
            FromEnv::Ty(ty) => self.visit_ty(ty),
        }
    }

    fn visit_domain_goal(&mut self, domain_goal: &DomainGoal<I>) {
        debug!(
            "EnvElaborator::visit_domain_goal(domain_goal={:?})",
            domain_goal
        );
        match domain_goal {
            DomainGoal::FromEnv(from_env) => self.visit_from_env(from_env),
            _ => {}
        }
    }

    fn visit_program_clause(&mut self, clause: &ProgramClause<I>) {
        debug!("visit_program_clause(clause={:?})", clause);
        match clause {
            ProgramClause::Implies(clause) => self.visit_domain_goal(&clause.consequence),
            ProgramClause::ForAll(clause) => self.visit_domain_goal(&clause.value.consequence),
        }
    }
}
