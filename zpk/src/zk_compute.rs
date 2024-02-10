/// Template zk computation. Computes the sum of the secret variables.
use pbc_zk::*;

pub fn zk_compute() -> Sbi32 {
      let mut sum: Sbi32 = sbi32_from(0);

      // Sum each variable
      for variable_id in 1..(num_secret_variables() + 1) {
          sum = sum + sbi32_input(variable_id);
      }

      sum
}
