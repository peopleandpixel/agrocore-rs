mod repositories;

pub use repositories::{SiteRepo, OrderRepo, UserRepo, TaskDataRepo};

use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::{Client, Collection, IndexModel};
use mongodb::bson::doc;

#[derive(Clone)]
pub struct Database {
    client: Client,
    db_name: String,
}

impl Database {
    pub async fn connect(uri: &str, db_name: &str) -> anyhow::Result<Self> {
        let mut opts = ClientOptions::parse(uri).await?;
        opts.max_pool_size = Some(100);
        opts.min_pool_size = Some(10);
        opts.max_idle_time = Some(std::time::Duration::from_secs(300));
        opts.connect_timeout = Some(std::time::Duration::from_secs(10));
        opts.server_selection_timeout = Some(std::time::Duration::from_secs(5));
        let client = Client::with_options(opts)?;
        let db = Self { client, db_name: db_name.to_string() };
        db.create_indexes().await?;
        Ok(db)
    }

    async fn create_indexes(&self) -> anyhow::Result<()> {
        // Users: unique email
        let user_collection = self.collection::<agrocore_domain::entities::user::User>("users");
        user_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "email": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        ).await?;

        // Sites: tenant_id + label
        let site_collection = self.collection::<agrocore_domain::entities::site::Site>("sites");
        site_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "tenant_id": 1, "label": 1 })
                .build(),
        ).await?;

        // Orders: tenant_id + status + deadline_date
        let order_collection = self.collection::<agrocore_domain::entities::order::Order>("orders");
        order_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "tenant_id": 1, "status": 1, "deadline_date": 1 })
                .build(),
        ).await?;

        // Task Data: tenant_id + worker_id + started_at
        let task_collection = self.collection::<agrocore_domain::entities::task::TaskData>("task_data");
        task_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "tenant_id": 1, "worker_id": 1, "started_at": -1 })
                .build(),
        ).await?;

        Ok(())
    }

    pub fn db(&self) -> mongodb::Database {
        self.client.database(&self.db_name)
    }

    pub fn collection<T: Send + Sync>(&self, name: &str) -> Collection<T> {
        self.db().collection(name)
    }

    pub fn site_repo(&self) -> SiteRepo {
        SiteRepo::new(self.collection("sites"))
    }

    pub fn order_repo(&self) -> OrderRepo {
        OrderRepo::new(self.collection("orders"))
    }

    pub fn user_repo(&self) -> UserRepo {
        UserRepo::new(self.collection("users"))
    }

    pub fn task_data_repo(&self) -> TaskDataRepo {
        TaskDataRepo::new(self.collection("task_data"))
    }
}
