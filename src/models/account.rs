use crate::errors::*;
use crate::fmt::colors::*;
use diesel;
use diesel::prelude::*;
use crate::models::*;
use chrono::NaiveDateTime;


#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="accounts"]
pub struct Account {
    pub id: i32,
    pub value: String,
    pub service: String,
    pub username: String,
    pub displayname: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
    pub unscoped: bool,
}

impl Model for Account {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::accounts::dsl::*;

        let results = accounts.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::accounts::dsl::*;

        let query = accounts.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter_with_param(db: &Database, filter: &Filter, param: &str) -> Result<Vec<Self>> {
        use crate::schema::accounts::dsl::*;

        let query = accounts
            .filter(service.eq(param))
            .filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::accounts::dsl::*;

        diesel::delete(accounts.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::accounts::dsl::*;

        diesel::delete(accounts.filter(id.eq(my_id)))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn value(&self) -> &Self::ID {
        &self.value
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use crate::schema::accounts::dsl::*;

        let domain = accounts.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::accounts::dsl::*;

        let domain = accounts.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::accounts::dsl::*;

        let domain = accounts.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(domain)
    }
}

impl Scopable for Account {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::accounts::dsl::*;

        diesel::update(accounts.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::accounts::dsl::*;

        diesel::update(accounts.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

pub struct PrintableAccount {
    value: String,
}

impl fmt::Display for PrintableAccount {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintableAccount> for Account {
    fn printable(&self, _db: &Database) -> Result<PrintableAccount> {
        Ok(PrintableAccount {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedAccount {
    id: i32,
    value: String,
    displayname: Option<String>,
    email: Option<String>,
    url: Option<String>,
    last_seen: Option<NaiveDateTime>,
    unscoped: bool,
}

impl DisplayableDetailed for DetailedAccount {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.debug::<Green, _>(&self.value)?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.displayname)?;
        w.opt_debug::<Yellow, _>(&self.email)?;
        w.opt_debug::<Yellow, _>(&self.url)?;
        w.opt_debug::<Yellow, _>(&self.last_seen)?;
        w.end_group()?;

        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::DetailFormatter) -> fmt::Result {
        Ok(())
    }
}

display_detailed!(DetailedAccount);

impl Detailed for Account {
    type T = DetailedAccount;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        Ok(DetailedAccount {
            id: self.id,
            value: self.value.to_string(),
            displayname: self.displayname.clone(),
            email: self.email.clone(),
            url: self.url.clone(),
            last_seen: self.last_seen.clone(),
            unscoped: self.unscoped,
        })
    }
}

#[derive(Insertable)]
#[table_name="accounts"]
pub struct NewAccount<'a> {
    pub value: &'a str,
    pub service: &'a str,
    pub username: &'a str,
    pub displayname: Option<&'a String>,
    pub email: Option<&'a String>,
    pub url: Option<&'a String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl<'a> InsertableStruct<Account> for NewAccount<'a> {
    fn value(&self) -> &str {
        self.value
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(accounts::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl<'a> Upsertable<Account> for NewAccount<'a> {
    type Update = AccountUpdate;

    fn upsert(self, existing: &Account) -> Self::Update {
        Self::Update {
            id: existing.id,
            displayname: Self::upsert_str(self.displayname, &existing.displayname),
            email: Self::upsert_str(self.email, &existing.email),
            url: Self::upsert_str(self.url, &existing.url),
            last_seen: Self::upsert_opt(self.last_seen, &existing.last_seen),
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="accounts"]
pub struct NewAccountOwned {
    pub value: String,
    pub service: String,
    pub username: String,
    pub displayname: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl Printable<PrintableAccount> for NewAccountOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableAccount> {
        Ok(PrintableAccount {
            value: self.value.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertAccount {
    pub service: String,
    pub username: String,
    pub displayname: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl LuaInsertToNewOwned for InsertAccount {
    type Target = NewAccountOwned;

    fn try_into_new(self) -> Result<NewAccountOwned> {
        if self.service.contains('/') {
            bail!("Service field can't contain `/`");
        }
        let value = format!("{}/{}", self.service, self.username);
        Ok(NewAccountOwned {
            value,
            service: self.service,
            username: self.username,
            displayname: self.displayname,
            email: self.email,
            url: self.url,
            last_seen: self.last_seen,
        })
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="accounts"]
pub struct AccountUpdate {
    pub id: i32,
    pub displayname: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
}

impl Upsert for AccountUpdate {
    fn is_dirty(&self) -> bool {
        self.displayname.is_some() ||
        self.email.is_some() ||
        self.url.is_some() ||
        self.last_seen.is_some()
    }

    fn generic(self) -> Update {
        Update::Account(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_account(&self)
    }
}

impl Updateable<Account> for AccountUpdate {
    fn changeset(&mut self, existing: &Account) {
        Self::clear_if_equal(&mut self.displayname, &existing.displayname);
        Self::clear_if_equal(&mut self.email, &existing.email);
        Self::clear_if_equal(&mut self.url, &existing.url);
        Self::clear_if_equal(&mut self.last_seen, &existing.last_seen);
    }

    fn fmt(&self, updates: &mut Vec<String>) {
        Self::push_value(updates, "displayname", &self.displayname);
        Self::push_value(updates, "email", &self.email);
        Self::push_value(updates, "url", &self.url);
        Self::push_value(updates, "last_seen", &self.last_seen);
    }
}
