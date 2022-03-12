use std::env;
use std::io::Read;

use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use lazy_static::lazy_static;
use tokio::io::AsyncReadExt;
use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher, RecommendedWatcher};
use tokio::fs::{DirEntry, File};
use tokio::sync::{RwLock, RwLockReadGuard};
use visdom::Vis;
use serde::{Deserialize, Serialize};
use async_recursion::async_recursion;
use log::info;


lazy_static! {
    static ref CONFIG_DATA: RwLock<Vec<Config>> = RwLock::new(vec![]);
}

/// 资源核心类
#[derive(Clone)]
pub struct Resource {
    /// 本地监听路径.
    pub local_path: String,
    /// 配置路径.
    pub config_path: String,
    /// 配置.
    config: &'static RwLock<Vec<Config>>,
}

impl Resource {
    /// 创建资源核心.
    pub async fn from(local_path: &str, config_path: &str) -> Resource {
        let resource = Resource {
            local_path: local_path.to_string(),
            config_path: config_path.to_string(),
            config: &CONFIG_DATA,
        };
        resource
    }

    /// 初始化配置.
    pub async fn init_config(&self) {
        let config_file_info = FileInfo::from(&self.config_path).await;
        if config_file_info.exist() {
            let config_list = serde_json::from_str(&config_file_info.read_string().await.unwrap()).unwrap();
            *self.config.write().await = config_list;
        } else {
            *self.config.write().await = vec![];
        }
    }

    /// 判断是否是静态资源文件.
    pub async fn is_static_file(&self, url: &str) -> Option<PathBuf> {
        let url = String::from(url)[1..].to_owned();
        if url.is_empty() {
            return None;
        }
        let path = Path::new(&self.local_path).join(url);
        if path.exists() {
            return Some(path.to_owned());
        }
        return None;
    }

    /// 是否有首页.
    pub async fn is_index(&self) -> bool {
        false
    }

    /// 根据配置解析首页模板.
    pub async fn parse_html_by_config(&self, uri: &str) -> String {
        let html = "".to_string();
        let config_list = &*self.config.read().await;
        let config = config_list.iter().find(|it| {
            match it.path.as_ref() {
                None => false,
                Some(value) => {
                    if value.starts_with("pre://") && value[6..].starts_with(&uri[1..]) {
                        return true;
                    }
                    if value.starts_with("tail://") && value[6..].ends_with(&uri[1..]) {
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
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
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
    pub async fn from(path: &str) -> Self {
        let mut path_buf = PathBuf::from(&FileInfo::parse_os_prefix(path));
        if path_buf.is_relative() {
            let path = path_buf.as_os_str().to_owned();
            path_buf = env::current_dir().unwrap();
            path_buf.push(&path);
        }
        FileInfo {
            path_string: path_buf.as_os_str().to_str().unwrap().to_string(),
            path_buf,
        }
    }

    /// 创建文件信息.
    pub async fn from_vec(path: Vec<&str>) -> Self {
        let mut path_buf = PathBuf::new();
        path.into_iter().for_each(|it| {
            path_buf.push(&FileInfo::parse_os_prefix(it));
        });
        if path_buf.is_relative() {
            let path = path_buf.as_os_str().to_owned();
            path_buf = env::current_dir().unwrap();
            path_buf.push(&path);
        }
        FileInfo {
            path_string: path_buf.as_os_str().to_str().unwrap().to_string(),
            path_buf,
        }
    }

    fn parse_os_prefix(path: &str) -> String {
        let result = path.to_owned();
        let mut result = result.as_str();
        loop {
            if result.starts_with(".") || result.starts_with("/") || result.starts_with("\\") {
                result = &result[1..]
            } else {
                break;
            }
        }
        result.to_string()
    }

    /// 文件是否存在.
    pub fn exist(&self) -> bool {
        Path::new(&self.path_buf.as_os_str()).exists()
    }

    /// 是否是文件.
    pub fn is_file(&self) -> bool {
        Path::new(&self.path_buf.as_os_str()).is_file()
    }

    /// 目录是否存在
    pub fn directory_exist(&self) -> bool {
        let path = self.path_buf.parent().unwrap();
        path.exists() && path.is_dir()
    }

    /// 创建目录
    pub async fn create_directory(&self) {
        let path = self.path_buf.parent().unwrap();
        tokio::fs::create_dir_all(path).await;
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


// // 首页内容
//         let index_file_info = FileInfo::from_vec(vec![&self.local_path, "index.html"]).await;
//         if index_file_info.exist() {
//             *self.index_data.write().await = index_file_info.read_string().await.unwrap();
//         } else {
//             *self.index_data.write().await = String::new();
//         }
//
//         // 静态资源文件索引
//         let path_list = &self.scan_directory(&self.local_path).await;
//         if path_list.len() > 0 {
//             let mut uri_list = vec![];
//             for it in path_list {
//                 match it.to_str() {
//                     None => continue,
//                     Some(value) => uri_list.push(value[self.local_path.len() + 1..].to_owned())
//                 }
//             }
//             *self.file_list.write().await = uri_list;
//         }


//
// /// 扫描目录.
// #[async_recursion]
// async fn scan_directory(&self, local_path: &str) -> Vec<PathBuf> {
//     let mut paths = tokio::fs::read_dir(local_path).await.unwrap();
//     let mut path_list = vec![];
//     loop {
//         let item = paths.next_entry().await.unwrap();
//         match item {
//             None => break,
//             Some(value) => {
//                 let path_buf = value.path();
//                 let path = path_buf.as_path();
//                 if path.is_dir() {
//                     let directory_path = path_buf.to_str().unwrap();
//                     path_list.append(&mut self.scan_directory(directory_path).await)
//                 }
//                 if path.is_file() {
//                     path_list.push(path_buf)
//                 }
//             }
//         }
//     }
//     path_list
// }
//
// /// 更新文件.
// fn update_file(&self, event: notify::op::Op, path: PathBuf) {
//     let uri = match path.as_os_str().to_str() {
//         None => return,
//         Some(value) => value[self.local_path.len() + 1..].to_owned()
//     };
//     if event.contains(notify::op::CREATE) {
//         tokio::spawn(async move {
//             println!("{:?}", FILE_LIST.read().await);
//             FILE_LIST.write().await.push(uri);
//         });
//     }
//     if event.contains(notify::op::REMOVE) {
//         //(*self.file_list.write().await).retain(|&it| it != uri);
//     }
//     if event.contains(notify::op::RENAME) {
//         // TODO
//     }
// }