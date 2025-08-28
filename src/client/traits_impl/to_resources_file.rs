use crate::client::structs::raw_file_xml::{
    CurrentUserPrivilegeSet, MultiStatus, Prop, PropStat, Response,
};
use crate::client::traits::to_resources_file::ToResourcesFile;
use crate::resources_file::ResourcesFile;
use reqwest::Url;

/**
pub struct ResourcesFile {
    pub relative_root_path: String, // 文件的相对路径（相对根目录）
    pub absolute_path: String,      // 文件的完整路径（从 href 拿到）
    pub name: String,               // 友好化的文件或目录名
    pub is_dir: bool,               // 是否目录
    pub size: Option<u64>,          // 文件大小（字节）
    pub size_str: Option<String>,   // 格式化后的大小，比如 "12.3MB"
    pub last_modified: Option<DateTime<FixedOffset>>, // 原始时间
    pub mime: Option<String>,       // MIME 类型
    pub owner: Option<String>,      // 所有者
    pub etag: Option<String>,       // 清理后的 ETag
    pub privileges: Vec<String>,    // 权限列表
}
**/

fn take_ok_propstat(propstats: Vec<PropStat>) -> Option<PropStat> {
    // 从 propstats 中拿到第一个 HTTP 状态是 2xx 的 PropStat（直接 move 出来）
    propstats.into_iter().find(|ps| {
        ps.status
            .split_whitespace()
            .find_map(|t| t.parse::<u16>().ok())
            .map(|code| (200..=299).contains(&code))
            .unwrap_or(false)
    })
}

fn decode_name(display_name: Option<String>, href: &str) -> String {
    // 如果服务端给了 display_name 就直接用（move），否则从 href 末尾提取文件名并 URL 解码
    display_name.unwrap_or_else(|| {
        percent_encoding::percent_decode_str(
            href.trim_end_matches('/').rsplit('/').next().unwrap_or(""),
        )
        .decode_utf8_lossy()
        .to_string()
    })
}

fn extract_privileges(
    cups: Option<CurrentUserPrivilegeSet>,
) -> Vec<String> {
    // 从权限对象中提取权限标识（直接消耗数据避免 clone）
    match cups {
        Some(set) => set
            .privileges
            .into_iter()
            .flat_map(|pr| {
                let mut v = Vec::new();
                if pr.read.is_some() {
                    v.push("read".to_string());
                }
                if pr.write.is_some() {
                    v.push("write".to_string());
                }
                if pr.all.is_some() {
                    v.push("all".to_string());
                }
                if pr.read_acl.is_some() {
                    v.push("read_acl".to_string());
                }
                if pr.write_acl.is_some() {
                    v.push("write_acl".to_string());
                }
                v
            })
            .collect(),
        None => Vec::new(),
    }
}

fn clean_etag(raw: Option<String>) -> Option<String> {
    // 去掉 ETag 的首尾引号以及多余空格
    raw.map(|s| s.trim().trim_matches('"').to_string())
}

impl ToResourcesFile for MultiStatus {
    fn to_resources_files(
        self,
        base_url: &Url,
    ) -> Result<Vec<ResourcesFile>, String> {
        let mut resources = Vec::new();

        let mut iter = self.responses.into_iter();

        if iter.len() > 1 {
            iter.next(); // 丢掉第一个，因为第一个属于无用数据
        }

        // 消耗 multi_status.responses 中的每个 Response
        // 跳过第一项，一般第一项都属于请求的路径本身，属于脏数据
        for Response { href, propstats } in iter {
            // 挑选出第一个 2xx PropStat（消耗 propstats 避免 clone）
            let ok_ps = match take_ok_propstat(propstats) {
                Some(ps) => ps,
                None => continue, // 没有 2xx 状态就跳过
            };

            // 解构 PropStat，move 出 prop
            let PropStat { prop, .. } = ok_ps;

            // 再解构 Prop，move 出需要的字段
            let Prop {
                resource_type,
                content_length: size,
                last_modified,
                content_type: mime,
                display_name,
                owner,
                etag,
                current_user_privilege_set,
                ..
            } = prop;

            // 提前计算 name（因为等下 href 要被 move 进结构体）
            let name = decode_name(display_name, &href);

            // 判断是否目录
            let is_dir = resource_type
                .as_ref()
                .and_then(|rt| rt.is_collection.as_ref())
                .is_some();

            let absolute_path = base_url
                .join(&href)
                .map(|u| u.to_string())
                .unwrap_or_else(|_| href.clone());

            // 构造最终 FriendlyResource，绝大部分字段直接 move
            resources.push(ResourcesFile {
                relative_root_path: href, // move
                absolute_path,
                name, // 已提前生成
                is_dir,
                size,
                last_modified, // move
                mime,          // move
                owner,         // move
                etag: clean_etag(etag),
                privileges: extract_privileges(current_user_privilege_set),
            });
        }

        Ok(resources)
    }
}
