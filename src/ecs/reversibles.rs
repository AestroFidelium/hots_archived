// use crate::support::*;
// use bevy_ecs::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Change<T> {
    timestamp: Instant,
    value_after: T,
}
#[derive(Debug, Clone, Default)]
pub struct Reversible<T> {
    value: T,
    history_of_changes: VecDeque<Change<T>>,
}

impl<T: PartialEq> PartialEq<T> for Reversible<T> {
    fn eq(&self, other: &T) -> bool {
        &self.value == other
    }
}

impl<
    T: std::ops::AddAssign<T>
        + std::ops::SubAssign<T>
        + std::ops::MulAssign<T>
        + std::ops::DivAssign<T>
        + Clone,
> Reversible<T>
{
    pub fn new(value: T) -> Self {
        Self {
            value,
            history_of_changes: VecDeque::new(),
        }
    }

    pub fn set_value(&mut self, value: T) {
        self.history_of_changes.push_back(Change {
            timestamp: std::time::Instant::now(),
            value_after: value.clone(),
        });
        self.value = value;
    }

    pub fn set_value_no_history(&mut self, value: T) {
        self.value = value;
    }

    pub fn value(&self) -> T {
        self.value.clone()
    }

    pub fn get_value_seconds_ago(&self, duration: Duration) -> T {
        let cutoff = Instant::now() - duration;

        for change in self.history_of_changes.iter().rev() {
            if change.timestamp <= cutoff {
                return change.value_after.clone();
            }
        }

        self.value.clone()
    }

    pub fn iter_values_since(&self, duration: Duration) -> impl Iterator<Item = &T> {
        let cutoff = Instant::now() - duration;

        self.history_of_changes
            .iter()
            .rev()
            .filter(move |change| change.timestamp >= cutoff)
            .map(|change| &change.value_after)
    }

    pub fn values_during(&self, duration: Duration) -> Vec<T> {
        let cutoff = Instant::now() - duration;

        let result: Vec<T> = self
            .history_of_changes
            .iter()
            .filter(|change| change.timestamp >= cutoff)
            .map(|change| change.value_after.clone())
            .collect();

        result
    }

    pub fn values_during_rev(&self, duration: Duration) -> Vec<T> {
        let cutoff = Instant::now() - duration;

        let mut result: Vec<T> = self
            .history_of_changes
            .iter()
            .filter(|change| change.timestamp >= cutoff)
            .map(|change| change.value_after.clone())
            .collect();

        result.reverse();

        result
    }
}

impl<
    T: std::ops::AddAssign<T>
        + std::ops::SubAssign<T>
        + std::ops::MulAssign<T>
        + std::ops::DivAssign<T>
        + Clone,
> From<T> for Reversible<T>
{
    fn from(value: T) -> Self {
        Reversible::new(value)
    }
}

macro_rules! impl_reversible_assign {
    ($trait:ident, $method:ident, $enum:ident) => {
        impl<T> std::ops::$trait<T> for Reversible<T>
        where
            T: std::ops::$trait<T>
                + std::ops::AddAssign<T>
                + std::ops::SubAssign<T>
                + std::ops::MulAssign<T>
                + std::ops::DivAssign<T>
                + Clone,
        {
            fn $method(&mut self, rhs: T) {
                self.value.$method(rhs.clone());
                self.history_of_changes.push_back(Change {
                    timestamp: std::time::Instant::now(),
                    value_after: self.value.clone(),
                });
            }
        }
    };
}

impl_reversible_assign!(AddAssign, add_assign, Add);
impl_reversible_assign!(SubAssign, sub_assign, Sub);
impl_reversible_assign!(MulAssign, mul_assign, Mul);
impl_reversible_assign!(DivAssign, div_assign, Div);
