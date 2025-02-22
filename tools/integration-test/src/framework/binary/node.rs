/*!
   Constructs for running test cases with two full nodes
   running without setting up the relayer.
*/

use toml;

use crate::bootstrap::single::bootstrap_single_node;
use crate::chain::builder::ChainBuilder;
use crate::error::Error;
use crate::framework::base::HasOverrides;
use crate::framework::base::{run_basic_test, BasicTest};
use crate::types::config::TestConfig;
use crate::types::single::node::FullNode;

/**
   Runs a test case that implements [`BinaryNodeTest`].
*/
pub fn run_binary_node_test<Test, Overrides>(test: &Test) -> Result<(), Error>
where
    Test: BinaryNodeTest,
    Test: HasOverrides<Overrides = Overrides>,
    Overrides: NodeConfigOverride,
{
    run_basic_test(&RunBinaryNodeTest { test })
}

/**
   This trait is implemented for test cases that need to have two full nodes
   running without the relayer being setup.

   The test case is given two [`FullNode`] which represents the two running full nodes.

   Test writers can use this to implement more advanced test cases which
   require manual setup of the relayer, so that the relayer can be started
   and stopped at a suitable time within the test.
*/
pub trait BinaryNodeTest {
    /// Test runner
    fn run(&self, config: &TestConfig, node_a: FullNode, node_b: FullNode) -> Result<(), Error>;
}

/**
   An internal trait that can be implemented by test cases to override the
   full node config before the chain gets initialized.

   The config is in the dynamic-typed [`toml::Value`] format, as we do not
   want to model the full format of the node config in Rust. Test authors
   can use the helper methods in [`chain::config`](crate::chain::config)
   to modify common config fields.

   This is called by [`RunBinaryNodeTest`] before the full nodes are
   initialized and started.

   Test writers should implement
   [`TestOverrides`](crate::framework::overrides::TestOverrides)
   for their test cases instead of implementing this trait directly.
*/
pub trait NodeConfigOverride {
    /// Modify the full node config
    fn modify_node_config(&self, config: &mut toml::Value) -> Result<(), Error>;
}

/**
   A wrapper type that lifts a test case that implements [`BinaryNodeTest`]
   into a test case that implements [`BasicTest`].
*/
pub struct RunBinaryNodeTest<'a, Test> {
    /// Inner test
    pub test: &'a Test,
}

impl<'a, Test, Overrides> BasicTest for RunBinaryNodeTest<'a, Test>
where
    Test: BinaryNodeTest,
    Test: HasOverrides<Overrides = Overrides>,
    Overrides: NodeConfigOverride,
{
    fn run(&self, config: &TestConfig, builder: &ChainBuilder) -> Result<(), Error> {
        let node_a = bootstrap_single_node(builder, "alpha", |config| {
            self.test.get_overrides().modify_node_config(config)
        })?;

        let node_b = bootstrap_single_node(builder, "beta", |config| {
            self.test.get_overrides().modify_node_config(config)
        })?;

        let _node_process_a = node_a.process.clone();
        let _node_process_b = node_b.process.clone();

        self.test.run(config, node_a, node_b)?;

        Ok(())
    }
}

impl<'a, Test> BinaryNodeTest for RunBinaryNodeTest<'a, Test>
where
    Test: BinaryNodeTest,
{
    fn run(&self, config: &TestConfig, node_a: FullNode, node_b: FullNode) -> Result<(), Error> {
        self.test
            .run(config, node_a, node_b)
            .map_err(config.hang_on_error())?;

        Ok(())
    }
}

impl<'a, Test, Overrides> HasOverrides for RunBinaryNodeTest<'a, Test>
where
    Test: HasOverrides<Overrides = Overrides>,
{
    type Overrides = Overrides;

    fn get_overrides(&self) -> &Self::Overrides {
        self.test.get_overrides()
    }
}
