use std::io::Read;

use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use lazy_static::lazy_static;
use tokio::io::AsyncReadExt;
use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher, RecommendedWatcher};
use tokio::fs::File;
use tokio::sync::{RwLock, RwLockReadGuard};
use visdom::Vis;
use crate::global;

lazy_static! {
    static ref FILE_LIST: RwLock<Vec<String>> = RwLock::new(vec![]);
    static ref INDEX_DATA: RwLock<String> = RwLock::new(String::new());
    static ref CONFIG_DATA: RwLock<Vec<Config>> = RwLock::new(vec![]);
}

/// 资源核心类
#[derive(Clone)]
pub struct Resource {
    /// 本地监听路径.
    pub local_path: String,
    /// 配置路径.
    pub config_path: String,
    /// 文件目录索引.
    file_list: &'static RwLock<Vec<String>>,
    /// 首页内容.
    index_data: &'static RwLock<String>,
    /// 配置.
    config: &'static RwLock<Vec<Config>>,
}

impl Resource {
    /// 创建资源核心.
    pub fn from(local_path: &str, config_path: &str) -> Resource {
        Resource {
            local_path: local_path.to_string(),
            config_path: config_path.to_string(),
            file_list: &FILE_LIST,
            index_data: &INDEX_DATA,
            config: &CONFIG_DATA,
        }
    }

    /// 执行监听文件夹 <b>阻塞线程</b>.
    pub fn watcher(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = raw_watcher(tx).unwrap();
        watcher.watch(&self.local_path, RecursiveMode::Recursive).unwrap();
        loop {
            match rx.recv() {
                Ok(RawEvent { path: Some(path), op: Ok(op), cookie }) => {
                    println!("{:?} {:?} ({:?})", op, path, cookie)
                }
                Ok(event) => println!("broken event: {:?}", event),
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }

    /// 资源更新处理.
    pub async fn handel(&self) {
        // 首页内容
        let index_file_info = FileInfo::from_vec(vec![&self.local_path, "index.html"]);
        if index_file_info.exist() {
            *self.index_data.write().await = index_file_info.read_string().await.unwrap();
        } else {
            *self.index_data.write().await = String::new();
        }

        // 配置文件
        let config_file_info = FileInfo::from(&self.config_path);
        if config_file_info.exist() {

        } else {
            *self.config.write().await = vec![];
        }
    }

    /// 检查路径是否命中文件索引.
    pub async fn check_url(&self, url: &str) -> Option<PathBuf> {
        if self.file_list.read().await.contains(&String::from(url)) {
            let mut path = PathBuf::from(&self.local_path);
            path.push(url);
            return Some(path);
        }
        return None;
    }

    /// 是否有首页.
    pub async fn is_index(&self) -> bool {
        self.index_data.read().await.len() > 0 || true
    }

    /// 根据配置解析首页模板.
    pub async fn parse_html_by_config(&self, uri: &str) -> String {
        let html = (*self.index_data.read().await).to_string();
        let config_list = self.config.read().await;
        let config = config_list.iter().find(|it| {
            match it.path.as_ref() {
                None => false,
                Some(value) => {
                    if value.starts_with("pre://") && value[6..0].starts_with(uri) {
                        return true;
                    }
                    if value.starts_with("tail://") && value[6..0].ends_with(uri) {
                        return true;
                    }
                    if value.starts_with("regular") {
                        return true;
                    }
                    false
                }
            }
        });
        if config.is_none() {
            return html;
        }

        let config = config.unwrap();
        let root = Vis::load(&html).unwrap();
        // 标题内容
        if config.title.is_some() {
            root.find("head > title").set_text(config.title.as_ref().unwrap());
        }
        // 资源
        if config.metas.is_some() {
            let mut heads = String::new();
            for it in config.metas.as_ref().unwrap() {
                heads.push_str(&format!(r##"<meta name="{}" content="{}">"##, it.name, it.content))
            }
            root.find("head").append(&mut Vis::load(&heads).unwrap());
        }
        // 标题
        if config.heads.is_some() {
            let mut heads = String::new();
            for it in config.heads.as_ref().unwrap() {
                heads.push_str(&format!("{}\n", it))
            }
            root.find("head").append(&mut Vis::load(&heads).unwrap());
        }
        root.html()
    }
}

/// 配置文件.
pub struct Config {
    /// 路径.
    pub path: Option<String>,
    /// 资源.
    pub metas: Option<Vec<ConfigMetas>>,
    /// 标题.
    pub title: Option<String>,
    /// 自定义头部.
    pub heads: Option<Vec<String>>,
}

/// 配置资源说明.
pub struct ConfigMetas {
    pub name: String,
    pub content: String,
}

/// 文件信息类.
pub struct FileInfo {
    /// 字符串路径.
    pub path_string: String,
    /// PathBuf路径.
    pub path_buf: PathBuf,
}

impl FileInfo {
    /// 创建文件信息.
    pub fn from(path: &str) -> Self {
        FileInfo {
            path_string: path.to_string(),
            path_buf: PathBuf::from(path),
        }
    }

    /// 创建文件信息.
    pub fn from_vec(path: Vec<&str>) -> Self {
        let mut path_buf = PathBuf::new();
        path.into_iter().for_each(|it| {
            path_buf.push(it);
        });
        FileInfo {
            path_string: path_buf.as_os_str().to_str().unwrap().to_string(),
            path_buf,
        }
    }

    /// 文件是否存在.
    pub fn exist(&self) -> bool {
        Path::new(&self.path_buf.as_os_str()).exists()
    }

    /// 是否是文件.
    pub fn is_file(&self) -> bool {
        Path::new(&self.path_buf.as_os_str()).is_file()
    }

    /// 读取文件到字符串.
    pub async fn read_string(&self) -> Result<String, FromUtf8Error> {
        let mut file = File::open(&self.path_buf).await.unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await;
        String::from_utf8(buffer)
    }
}


// pub async fn read_file(&self, url: &str) -> bool {
//     // let path = format!("/Users/xxscloud/code/test/rust/bbox/src/resource.rs");
//     // let mut async_file = tokio::fs::File::open("/Users/xxscloud/code/github/BlueberryBox/bbox/output.log").await.unwrap();
//
//     // let mut file = std::fs::File::open("/Users/xxscloud/code/github/BlueberryBox/bbox/output.log").unwrap();
//     // let mut buffer = [0u8; 16];
//     // let mut buffer2 = [0u8; 16];
//     // let len = async_file.read(&mut buffer).await.unwrap();
//     // let len = async_file.read(&mut buffer2).await.unwrap();
//     // println!("{}", len);
//     false
// }