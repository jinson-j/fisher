use faiss::{Index, index_factory, MetricType, index::IndexImpl};

pub struct VectorStore {
    index: IndexImpl,
    dim: usize,
}

impl VectorStore {
    pub fn new(dim: usize) -> faiss::error::Result<Self> {
        let index = index_factory(dim as u32, "Flat", MetricType::L2)?;
        Ok(VectorStore { index, dim })
    }
    pub fn add(&mut self, vectors: &[Vec<f32>]) -> faiss::error::Result<()> {
        for v in vectors {
            assert_eq!(v.len(), self.dim, "Vector has wrong dimension");
            self.index.add(&v)?;
        }
        Ok(())
    }

    pub fn query(&mut self, query: &[f32], k: usize) -> faiss::error::Result<(Vec<f32>, Vec<faiss::Idx>)> {
        assert_eq!(query.len(), self.dim, "Query vector has wrong dimension");
        let result = self.index.search(query, k)?;
        Ok((result.distances, result.labels))
    }

    /// Get the number of vectors in the store.
    pub fn len(&self) -> usize {
        self.index.ntotal() as usize
    }
}

