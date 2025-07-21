# 讯飞听见

## 录音笔渠道转写接口

以下接口均基于录音笔B1测试。

### 发起转写任务

- Windows
  - `POST: https://www.iflyrec.com/XFTJPCAdaptService/v1/B1/orders/`
- Android
  - `POST: https://www.iflyrec.com/XFTJAppAdaptService/v1/parrotA1/orders`

利用 `fileId` 获取 `orderId`

### 上报音频文件

`POST: https://www.iflyrec.com/AudioStreamService/v1/audios`

此接口在两平台客户端一致。

上传的内容是音频信息，包含：

- 名称
- 路径
- 时长
- `fileId` 置空字符串
- ...

存在参数 `type=block`，猜测为分块上传标志。

用于获取音频的 `fileId`。

### 上传音频文件

`POST: https://www.iflyrec.com/AudioStreamService/v1/audios`

此接口在两平台客户端一致。

上传的音频是源音频经过简单处理后的版本，~~操作包括去除了源文件头部并~~（仅Windows版去除，Android版为直接拼接）在文件头部加入音频文件报告接口使用的信息，其中 `fileId` 替换为音频文件报告接口获取到的 `fileId`。

存在参数 `type=block`，猜测为分块上传标志。

### 计算音频时长

`POST: https://www.iflyrec.com/TranscriptOrderService/v1/tempAudios/62ff6ac25d8746bcb3930e925fb0c2e6/calculateDuration`

未检查Android版。

### 获取转写结果

- Windows
  - `GET: https://www.iflyrec.com/XFTJWebAdaptService/v1/hyjy/PPmz2507192316CB9D878600002/transcriptResults/16`
  - 有参数：`fileSource=app&originAudioId=390741385638084610&t=1752938543587`
    - `PPmz2507192316CB9D878600002` 应该为转写任务ID（ `orderId` ）
      `390741385638084610` 也叫 `transcriptId`
- Android
  - `GET:https://www.iflyrec.com/XFTJAppAdaptService/v4/audios/390937550981009416/transcriptResults`
  - 有参数：`resultType=16&fileSource=app`

### 获取当前账号所有任务

`POST: https://www.iflyrec.com/XFTJWebAdaptService/v2/hjProcess/recentOperationFiles`

Android版暂未找到该接口。

存在参数 `t=1752977303593`，猜测为当前时间戳。

接口返回的任务信息有：

- `orderId`
- `originAudioId`
- `orderStatus`
- `audioDurations`
- `fileId`
- ...

### 获取支持转写的目标语言

- Windows
  - `GET: https://www.iflyrec.com/XFTJWebAdaptService/v1/hyjy/supportedLanguages`
  - 存在参数 `showAutoDialect=true&t=1752977308602`
- Android
  - `GET: https://www.iflyrec.com/XFTJAppAdaptService/v1/parrotA1/supportedLanguages`
  - 存在参数 `type=A1NonRealtime`

### 获取可选的专业领域

`GET: https://www.iflyrec.com/XFTJWebAdaptService/v1/domainList`

## 当前任务

- [ ] 查看各端口的安卓版本