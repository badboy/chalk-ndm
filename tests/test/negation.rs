//! Tests related to `not { }` goals.

use super::*;

#[test]
fn simple_negation() {
    test! {
        program {
            struct i32 {}
            trait Foo {}
        }

        goal {
            not { i32: Foo }
        } yields {
            "Unique"
        }

        goal {
            not {
                not { i32: Foo }
            }
        } yields {
            "No"
        }

        goal {
            not {
                not {
                    not { i32: Foo }
                }
            }
        } yields {
            "Unique"
        }

        goal {
            exists<T> {
                not { T: Foo }
            }
        } yields {
            "Ambig"
        }

        goal {
            forall<T> {
                not { T: Foo }
            }
        } yields {
            "Unique"
        }

        goal {
            not {
                exists<T> { T: Foo }
            }
        } yields {
            "Unique"
        }

        goal {
            not {
                forall<T> { T: Foo }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn deep_negation() {
    test! {
        program {
            struct Foo<T> {}
            trait Bar {}
            trait Baz {}

            impl<T> Bar for Foo<T> where T: Baz {}
        }

        goal {
            not {
                exists<T> { T: Baz }
            }
        } yields {
            "Unique"
        }

        goal {
            not {
                exists<T> { Foo<T>: Bar }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn negation_quantifiers() {
    test! {
        program {
            struct i32 {}
            struct u32 {}
        }

        goal {
            not {
                forall<T, U> {
                    T = U
                }
            }
        } yields {
            "Unique"
        }

        goal {
            not {
                exists<T, U> {
                    T = U
                }
            }
        } yields {
            "No"
        }

        goal {
            forall<T, U> {
                not {
                    T = U
                }
            }
        } yields {
            "No"
        }
    }
}

#[test]
fn negation_free_vars() {
    test! {
        program {
            struct Vec<T> {}
            struct i32 {}
            struct u32 {}
            trait Foo {}
            impl Foo for Vec<u32> {}
        }

        goal {
            exists<T> {
                not { Vec<T>: Foo }
            }
        } yields {
            "Ambig"
        }
    }
}

#[test]
fn negation_assoc_types_cardinality_1() {
    test! {
        program {
            trait Iterator {
                type Item;
            }
        }

        goal {
            forall<T, U> {
                if (T: Iterator) {
                    not {
                        <T as Iterator>::Item = U
                    }
                }
            }
        } yields {
            "No"
        }

        goal {
            forall<T, U> {
                if (T: Iterator; U: Iterator) {
                    not {
                        <T as Iterator>::Item = <U as Iterator>::Item
                    }
                }
            }
        } yields {
            "No"
        }
    }
}

#[test]
#[ignore] // FIXME
fn negation_assoc_types_cardinality_2() {
    test! {
        program {
            trait Foo<X> {
                type Bar;
            }


            struct I32 { }
            struct U32 { }
        }

        goal {
            forall<T> {
                if (forall<U> { T: Foo<U> }) {
                    not {
                        <T as Foo<I32>>::Bar = <T as Foo<U32>>::Bar
                    }
                }
            }
        } yields {
            "No"
        }
    }
}
