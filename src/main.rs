use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};


// 创建文件结果
#[derive(Debug)]
enum CreateFileResult {
    // 无法获取accessToken
    NoAccessToken,
    // 创建目录失败
    CreateFileResult,
    // 读取json数据出错
    ReadJsonDataError,
}

use md5::{Digest, Md5};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::Value;

const API_URI: &str = "https://open-api.123pan.com";

// data.insert("clientID", "d983db1cf32f4788b7cce674df75ff33");
// data.insert("clientSecret", "1dfe6105f760447cb5df1292f44d035f");
#[derive(serde::Serialize)]
struct Baike123 {
    clientID: String,
    clientSecret: String,
}
#[derive(Deserialize)]
struct AccessTokenResponse {
    code: i32,
    message: String,
    data: Value,
    #[serde(rename = "x-traceID")]
    trace_id: String,
}

// 获取accessToken
pub async fn get_accessToken(baike123: Baike123) -> Result<String, String> {
    // 组装header
    let mut headers = HeaderMap::new();
    headers.insert("Platform", "open_platform".parse().unwrap());

    let client = reqwest::Client::new();
    let res: reqwest::RequestBuilder = client.post(API_URI.to_owned() + "/api/v1/access_token");
    match res.headers(headers).json(&baike123).send().await {
        Ok(resp) => {
            #[derive(Deserialize)]
            struct Data {
                accessToken: String,
                expiredAt: String,
            }
            let json_str = resp.text().await.unwrap();
            match serde_json::from_str::<AccessTokenResponse>(&json_str) {
                Ok(result) => match serde_json::from_value::<Data>(result.data) {
                    Ok(_access_token) => Ok(_access_token.accessToken),
                    Err(_err) => Err(_err.to_string()),
                },
                Err(_err) => Err(_err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

#[derive(serde::Serialize)]
struct Mkdir {
    // 目录名(注:不能重名)
    name: String,
    // 父目录id，上传到根目录时填写 0
    parentID: i64,
}
// 创建目录
pub async fn create_dir(access_token: String, mkdir: Mkdir) -> Result<i64, String> {
    let mut headers = HeaderMap::new();
    headers.insert("Platform", "open_platform".parse().unwrap());
    headers.insert("Authorization", access_token.parse().unwrap());

    let client = reqwest::Client::new();
    let res: reqwest::RequestBuilder = client.post(API_URI.to_owned() + "/upload/v1/file/mkdir");
    match res.headers(headers).json(&mkdir).send().await {
        Ok(resp) => {
            #[derive(Deserialize)]
            struct Data {
                dirID: i64,
            }
            let json_str = resp.text().await.unwrap();
            match serde_json::from_str::<AccessTokenResponse>(&json_str) {
                Ok(result) => match serde_json::from_value::<Data>(result.data) {
                    Ok(_data) => Ok(_data.dirID),
                    Err(_err) => Err(result.message),
                },
                Err(_err) => Err(_err.to_string()),
            }
        }
        Err(_err) => Err(_err.to_string()),
    }
}

#[derive(serde::Serialize)]
pub struct FileUpload {
    parentFileID: u64,
    filename: String,
    etag: String,
    size: u64,
}

/// let baike123 = Baike123 {
///     clientID: "".to_string(),
///     clientSecret: "".to_string(),
/// };
/// let access_token = get_accessToken(baike123).await.unwrap();
/// let file_path = Path::new("temp/317252433b407953d548cd934f35843b.jpg");
/// match upload_file(access_token, file_path, 10257529).await {
///     Ok(_) => {
///         println!("上传成功");
///     }
///     Err(_err) => {
///         println!("上传失败：{}", _err);
///     }
/// }
/// let dirID = create_dir(
///     access_token.clone(),
///     Mkdir {
///         name: "我大大大的文件夹".to_string(),
///         parentID: 0,
///     },
/// )
/// .await;
// 创建文件
pub async fn baike123_create_file(
    access_token: String,
    file_upload: FileUpload,
) -> Result<(std::string::String, i64), ()> {
    let mut headers = HeaderMap::new();
    headers.insert("Platform", "open_platform".parse().unwrap());
    headers.insert("Authorization", access_token.parse().unwrap());

    let client = reqwest::Client::new();
    let res: reqwest::RequestBuilder = client.post(API_URI.to_owned() + "/upload/v1/file/create");

    match res.headers(headers).json(&file_upload).send().await {
        Ok(resp) => {
            #[derive(Deserialize)]
            struct Data {
                fileID: i64,
                // 预上传ID
                preuploadID: String,
                reuse: bool,
                sliceSize: i64,
            }
            let json_str = resp.text().await.unwrap();
            match serde_json::from_str::<AccessTokenResponse>(&json_str) {
                Ok(result) => match serde_json::from_value::<Data>(result.data) {
                    Ok(data) => {
                        if data.reuse {
                            Err(())
                        } else {
                            Ok((data.preuploadID, data.sliceSize))
                        }
                    }
                    Err(_err) => {
                        panic!("{:?}", result.message)
                    }
                },
                Err(_err) => panic!("{:?}", _err),
            }
        }
        Err(_err) => panic!("{:?}", _err),
    }
}

#[derive(serde::Serialize)]
pub struct PreUploadInfo {
    // 预上传ID
    preuploadID: String,
    // 分片序号，从1开始自增
    slice_no: u64,
}

// 获取上传地址
pub async fn get_upload_url(
    access_token: String,
    pre_upload_info: PreUploadInfo,
) -> Result<String, String> {
    let mut headers = HeaderMap::new();
    headers.insert("Platform", "open_platform".parse().unwrap());
    headers.insert("Authorization", access_token.parse().unwrap());

    let client = reqwest::Client::new();
    let res: reqwest::RequestBuilder =
        client.post(API_URI.to_owned() + "/upload/v1/file/get_upload_url");
    match res.headers(headers).json(&pre_upload_info).send().await {
        Ok(resp) => {
            #[derive(Deserialize)]
            struct Data {
                presignedURL: String,
            }
            let json_str = resp.text().await.unwrap();
            match serde_json::from_str::<AccessTokenResponse>(&json_str) {
                Ok(result) => match serde_json::from_value::<Data>(result.data) {
                    Ok(data) => Ok(data.presignedURL),
                    Err(_err) => Err(result.message),
                },
                Err(_err) => Err(_err.to_string()),
            }
        }
        Err(_err) => Err(_err.to_string()),
    }
}
#[derive(Clone, serde::Serialize)]
struct UploadSliceData {
    preuploadID: String,
}

/// 列举已上传分片
pub async fn pre_upload_slice_complete(
    access_token: String,
    upload_slice_data: UploadSliceData,
) -> Result<Vec<String>, String> {
    let mut headers = HeaderMap::new();
    headers.insert("Platform", "open_platform".parse().unwrap());
    headers.insert("Authorization", access_token.parse().unwrap());

    let client = reqwest::Client::new();
    let res: reqwest::RequestBuilder =
        client.post(API_URI.to_owned() + "/upload/v1/file/list_upload_parts");
    match res.headers(headers).json(&upload_slice_data).send().await {
        Ok(resp) => {
            #[derive(Debug, Deserialize)]
            pub struct PartInfo {
                part_number: u64,
                size: u64,
                etag: String,
            }
            #[derive(Debug, Deserialize)]
            pub struct Parts {
                parts: Vec<PartInfo>,
            }
            let json_str = resp.text().await.unwrap();
            println!("json_str: {}", json_str);
            match serde_json::from_str::<AccessTokenResponse>(&json_str) {
                Ok(result) => {
                    let _data = serde_json::from_value::<Parts>(result.data).unwrap();
                    //返回etag
                    Ok(_data
                        .parts
                        .iter()
                        .map(|part| {
                            // println!("part_number: {}, size: {}, etag: {}", part.part_number, part.size, part.etag);
                            part.etag.clone()
                        })
                        .collect::<Vec<String>>())
                }
                Err(_err) => Err(_err.to_string()),
            }
        }
        Err(_err) => Err(_err.to_string()),
    }
}

// 上传完毕
pub async fn upload_complete(
    access_token: String,
    upload_slice_data: UploadSliceData,
) -> Result<bool, String> {
    let mut headers = HeaderMap::new();
    headers.insert("Platform", "open_platform".parse().unwrap());
    headers.insert("Authorization", access_token.parse().unwrap());

    let client = reqwest::Client::new();
    let res: reqwest::RequestBuilder =
        client.post(API_URI.to_owned() + "/upload/v1/file/upload_complete");
    match res.headers(headers).json(&upload_slice_data).send().await {
        Ok(resp) => {
            #[derive(Deserialize)]
            struct Data {
                fileID: Option<i64>,
                r#async: bool,
                completed: bool,
            }
            let json_str = resp.text().await.unwrap();
            match serde_json::from_str::<AccessTokenResponse>(&json_str) {
                Ok(result) => {
                    let _data = serde_json::from_value::<Data>(result.data).unwrap();
                    Ok(_data.completed)
                }
                Err(_err) => Err(_err.to_string()),
            }
        }
        Err(_err) => Err(_err.to_string()),
    }
}

// 上传文件
/// access_token: String, 鉴权token
/// file: File, 要上传的文件
/// dir_id: i64, 父目录id，上传到根目录时填写 0
pub async fn upload_file(
    access_token: String,
    file_path: &Path,
    dir_id: u64,
) -> Result<(), String> {
    // 尝试秒传
    let file: File = match File::open(file_path) {
        Ok(_file) => _file,
        Err(_err) => return Err(_err.to_string()),
    };

    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(_err) => return Err(_err.to_string()),
    };
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    match buf_reader.read_to_end(&mut contents) {
        Ok(_s) => (),
        Err(_err) => return Err(_err.to_string()),
    }

    // 获取文件md5
    let mut hasher = Md5::new();
    hasher.update(contents.clone());
    let result = hasher.finalize();
    let md5_str = format!("{:x}", result);

    // 创建文件
    let file_upload = FileUpload {
        parentFileID: dir_id,
        filename: file_path.file_name().unwrap().to_str().unwrap().to_string(),
        etag: md5_str.clone(),
        size: metadata.len(),
    };

    // 1，请求【创建文件】接口创建文件，接口返回的reuse为true时，表示秒传成功，上传结束。
    // 非秒传的情况将会返回预上传ID preuploadID 与分片大小 sliceSize,请将文件根据分片大小切分。
    // 创建文件
    let (preupload_id, slice_size) =
        match baike123_create_file(access_token.clone(), file_upload).await {
            Ok(d) => d,
            Err(_) => return Ok(()),
        };

    // 获取上传地址
    let pre_upload_info = PreUploadInfo {
        preuploadID: preupload_id.clone(),
        slice_no: 1,
    };
    let upload_url = get_upload_url(access_token.clone(), pre_upload_info).await?;
    match reqwest::Client::new()
        .put(upload_url)
        .body(contents.clone())
        .send()
        .await
    {
        Ok(_) => {
            let upload_slice_data = UploadSliceData {
                preuploadID: preupload_id.clone(),
            };
            if metadata.len() >= slice_size.try_into().unwrap() {
                let parts_etag =
                    pre_upload_slice_complete(access_token.clone(), upload_slice_data.clone())
                        .await?;
                for etag in parts_etag {
                    if etag == md5_str {
                        upload_complete(access_token.clone(), upload_slice_data.clone()).await?;
                    }
                }
            } else {
                upload_complete(access_token.clone(), upload_slice_data.clone()).await?;
            }
        }
        Err(_err) => return Err(_err.to_string()),
    }

    Ok(())
}
#[tokio::test]
async fn test() {

    // 10257529
}

fn main() {
    println!("Hello, world!");
}
