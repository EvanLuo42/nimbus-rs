use crate::rendering::context::RenderContext;
use crate::rendering::frame::FrameContext;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rayon::ThreadPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use crate::core::errors::NimbusError;

pub trait RenderPass: Send + Sync {
    fn name(&self) -> &str;
    
    fn dependencies(&self) -> &[&str];
    
    fn record_secondary(
        &self,
        frame: &FrameContext,
        ctx: &RenderContext
    ) -> Result<Arc<SecondaryAutoCommandBuffer>, NimbusError>;
}

#[derive(Default)]
pub struct RenderPassGraph {
    pub nodes: Vec<RenderPassNode>
}

impl RenderPassGraph {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_pass<P: RenderPass + 'static>(
        &mut self,
        name: impl Into<String>,
        dependencies: &[&str],
        pass: P
    ) {
        self.nodes.push(RenderPassNode {
            name: name.into(),
            pass: Arc::new(pass),
            dependencies: dependencies.iter().map(|s| s.to_string()).collect()
        })
    }

    pub fn execute_parallel_with_pool(
        &self,
        frame: &FrameContext,
        ctx: &RenderContext,
        pool: &ThreadPool,
    ) -> Vec<Arc<SecondaryAutoCommandBuffer>> {
        let mut visited = HashSet::new();
        let mut execution_order = vec![];
        
        fn dfs(
            name: &str,
            graph: &RenderPassGraph,
            visited: &mut HashSet<String>,
            output: &mut Vec<String>,
        ) {
            if visited.contains(name) {
                return;
            }

            let node = graph.nodes.iter().find(|n| n.name == name).unwrap();
            for dep in &node.dependencies {
                dfs(dep, graph, visited, output);
            }

            visited.insert(name.to_string());
            output.push(name.to_string());
        }

        for node in &self.nodes {
            dfs(&node.name, self, &mut visited, &mut execution_order);
        }
        
        let map: HashMap<String, &RenderPassNode> = self
            .nodes
            .iter()
            .map(|n| (n.name.clone(), n))
            .collect();
        
        pool.install(|| {
            execution_order
                .par_iter()
                .map(|name| {
                    let node = map.get(name).unwrap();
                    node.pass.record_secondary(frame, ctx).unwrap()
                })
                .collect()
        })
    }
}

pub struct RenderPassNode {
    pub name: String,
    pub pass: Arc<dyn RenderPass>,
    pub dependencies: Vec<String>
}
