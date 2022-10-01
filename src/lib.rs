use std::io;

use milli::{
    documents::{DocumentsBatchBuilder, DocumentsBatchReader},
    heed::EnvOpenOptions,
    update::{IndexDocuments, IndexDocumentsConfig, IndexerConfig},
    Index, Search,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Search Database using milli
pub struct Database {
    index: Index,
    config: IndexerConfig,
    indexing_config: IndexDocumentsConfig,
}

/// Constructors
impl Database {
    pub fn new() -> Self {
        let path = tempfile::tempdir().unwrap();
        let mut options = EnvOpenOptions::new();
        options.map_size(100 * 1024 * 1024);
        let index = Index::new(options, &path).unwrap();

        let config = IndexerConfig::default();
        let indexing_config = IndexDocumentsConfig::default();

        Self {
            index,
            config,
            indexing_config,
        }
    }
}

impl Database {
    pub fn add_document<'a, T: Serialize + Deserialize<'a>>(
        &mut self,
        document: T,
    ) -> Result<(), ()> {
        let mut wtxn = self.index.write_txn().unwrap();
        let mut documents_builders = DocumentsBatchBuilder::new(Vec::new());

        let value = serde_json::to_value(&document).unwrap();
        let object = value.as_object().unwrap();
        documents_builders.append_json_object(&object).unwrap();
        let vector = documents_builders.into_inner().unwrap();

        let content = DocumentsBatchReader::from_reader(io::Cursor::new(vector)).unwrap();

        let builder = IndexDocuments::new(
            &mut wtxn,
            &self.index,
            &self.config,
            self.indexing_config.clone(),
            |_| (),
        )
        .unwrap();

        let (builder, user_error) = builder.add_documents(content).unwrap();
        user_error.unwrap();
        builder.execute().unwrap();
        wtxn.commit().unwrap();

        Ok(())
    }

    pub fn add_documents<'a, T: Serialize + Deserialize<'a>, U: IntoIterator<Item = T>>(
        &mut self,
        documents: U,
    ) -> Result<(), ()> {
        for document in documents {
            self.add_document(document).ok();
        }
        Ok(())
    }
}

impl Database {
    pub fn search<T: Serialize + DeserializeOwned>(&self, query: &str) -> Result<Vec<T>, ()> {
        #[allow(unused_mut)]
        let mut rtxn = self.index.read_txn().unwrap();
        let mut search = Search::new(&rtxn, &self.index);
        search.query(query);
        search.limit(10);

        let ids = search.execute().unwrap().documents_ids;

        let items = self.index.documents(&rtxn, ids).unwrap();
        let fields_map = self.index.fields_ids_map(&rtxn).unwrap();
        let mut results: Vec<T> = Vec::new();
        for (_id, reader) in items.clone() {
            let mut map = serde_json::Map::new();
            for (field_id, field_value) in reader.iter() {
                let field_name = fields_map.name(field_id).unwrap();
                let json: serde_json::Value = serde_json::from_slice(field_value).unwrap();

                map.insert(field_name.to_string(), json);
            }
            let doc: T = serde_json::from_value(map.into()).unwrap();
            results.push(doc);
        }
        Ok(results)
    }

    fn kvreader_to_t<T: Serialize + DeserializeOwned>(&self, reader: obkv::KvReader<u16>) -> T {
        let rtxn = self.index.read_txn().unwrap();
        let fields_map = self.index.fields_ids_map(&rtxn).unwrap();
        let mut map = serde_json::Map::new();
        for (field_id, field_value) in reader.iter() {
            let field_name = fields_map.name(field_id).unwrap();
            let json: serde_json::Value = serde_json::from_slice(field_value).unwrap();

            map.insert(field_name.to_string(), json);
        }
        let doc: T = serde_json::from_value(map.into()).unwrap();
        doc
    }
}

impl Database {
    #[allow(dead_code)]
    pub fn get_document_by_external_id<T: Serialize + DeserializeOwned>(
        &self,
        external_id: usize,
    ) -> Option<T> {
        let rtxn = &self.index.read_txn().unwrap();
        let idio_map = &self.index.external_documents_ids(&rtxn).unwrap();

        if let Some(index) = idio_map.get(external_id.to_string()) {
            self.get_document_by_internal_id(index)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn get_document_by_internal_id<T: Serialize + DeserializeOwned>(
        &self,
        internal_id: u32,
    ) -> Option<T> {
        #[allow(unused_mut)]
        let mut rtxn = self.index.read_txn().unwrap();
        let results = self.index.documents(&rtxn, [internal_id]);
        match results {
            Ok(rs) => {
                let obj = rs[0];
                let _id = obj.0;
                let reader = obj.1;
                let doc: T = self.kvreader_to_t(reader);
                Some(doc)
            }
            Err(_) => None,
        }
    }
}
