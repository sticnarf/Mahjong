# Mahjong

本项目为一个麻将服务器，供AI比赛使用。本文档分为两部分，第一部分为此服务器使用的[麻将规则](#规则)，第二部分为交互的[接口规范](#交互规则)。

## 规则

为降低复杂度，本规则以国标麻将为基础进行了大量的简化并进行一定修改。

### 注意点

* 麻将牌共136张，即在国标麻将的基础上去掉了八张花牌。
* 一局定为4盘。去除了圈风、门风、庄家的概念。
* 没有牌城、掷骰与开牌的概念，AI拿到的牌全部直接由服务器发放。
* 整场比赛的第一盘时，第一个行牌的AI由服务器随机决定，从第二盘开始第一个行牌的AI为最近和牌的AI。
* 没有起和番数
* 轮到AI出牌的时候，AI必须在1秒内打出牌，否则以100毫秒1分进行罚分。
* 一方打出牌后，AI若要吃牌、碰牌、杠牌或是和牌，需要在0.5秒内向服务器发出指令，超时即无效。
* 测试共有6组，每组分为4局。不同组测试中AI相对位置不同。同组内四局使用的随机数种子不变，一局结束后每位AI的位置逆时针旋转一位。

### 番种

国标麻将的番种非常多，本规则将番种大大精简。

##### 88番

* 大四喜（不计碰碰和）
* 大三元（不计箭刻）
* 十三幺（不计五门齐、单钓将、门前清、混幺九）
* 绿一色（无发记清一色，*有发记混一色*）
* 四杠（暗杠另计）

##### 64番

* 小四喜
* 小三元（不计箭刻）
* 字一色（不计碰碰和）
* 四暗刻（不计门前清、碰碰和）
* 清幺九（不计碰碰和）

##### 48番

* 一色四同顺（不计一色三同顺、一般高）

##### 32番

* 三杠（暗杠另计）
* 混幺九（不计碰碰和）

##### 24番

* 七对（不计门前清、单钓将）
* 清一色
* 一色三同顺（不计一般高）

##### 16番

* 三同刻
* 三暗刻

##### 8番

* 三色三同顺

##### 6番

* 碰碰和
* 混一色
* 五门齐

##### 2番

* 门前清
* 断幺
* 平和
* 箭刻（可算多次）
* 暗杠（可算多次）

##### 1番

* 自摸
* 一般高（可算多次）
* 喜相逢（可算多次）
* 明杠（可算多次）
* 单钓将

### 记分规则

最开始每位AI的分数都是0分。底分为4分，基本分和牌后各个番种分数的总和。

若和牌方自摸，则另外三家都需付给和牌方（底分＋基本分）的分数。

否则，点炮方付给和牌方（底分＋基本分）的分数，其余两家付给和牌方底分的分数。

### 结束与排名

1. 当有任何一方AI程序意外退出时比赛结束，意外关闭的AI排名最后，其余AI按分数高低排名。
2. 当100盘牌局结束后比赛结束，AI按分数高低排名。

## 交互规则

服务器与AI通过标准输入和标准输出进行通信，每次通信的内容为一行不含多余空格json文本。

示例中的“标准输入”、“标准输出”都是以AI的角度上说的。“标准输入”中的内容即AI需要读取的内容，“标准输出”中的内容为AI需要打印的内容。

### 骨牌代号

* 字牌：东（E）、南（S）、西（W）、北（N）、中（Z）、发（F）、白（B）
* 万子（M）：1M、2M、3M、4M、5M、6M、7M、8M、9M
* 索子（S）：1S、2S、3S、4S、5S、6S、7S、8S、9S
* 筒子（T）：1T、2T、3T、4T、5T、6T、7T、8T、9T

~~PS：关于万的缩写的问题，是取自日语读音`man`…………………………~~

### 加入比赛

AI程序由服务器启动。AI先发送加入信息。

标准输出：

```json
{"type":"join"}
```

当四位AI都加入比赛之后，服务器会把id号发送给AI。

标准输入：

```json
{"type":"ready","id":3}
```

### 开始比赛

每盘牌局开始时，服务器会广播第一个行牌的AI的id。

标准输入：

```json
{"type":"start","first":3}
```

然后每个AI会收到13张起手牌。

标准输入：

```json
{"type":"init","tiles":["1S","1S","2T","5M","9S","4T","W","Z","1T","8M","9M","5S","6T"]}
```

### 行牌

要注意行牌是有时间限制的。

##### 拿牌

到AI拿牌的时候，服务器会将拿牌信息发给AI，同时向其他AI广播。

标准输入（拿牌AI）：

```json
{"type":"pick","tile":"5T"}
```

标准输入（其他AI）：

```json
{"type":"mpick","id":4}
```

##### 出牌

出牌需要AI在1秒内发送指令给服务器，然后服务器将出牌信息广播给其他AI。

标准输出（出牌AI）：

```json
{"type":"out","tile":"9M"}
```

标准输入（其他AI）：

```json
{"type":"mout","tile":"9M"}
```

这时三位AI有0.5秒的时间向服务器发出吃、碰、杠或和的指令。超过0.5秒的指令则无效。

由于吃的方式不唯一，所以吃的指令需要另外指定2张手中的牌。

标准输出（AI1）：

```json
{"type":"chi","tiles":["7M","8M"]}
```

标准输出（AI2）：

```json
{"type":"peng"}
```

当吃和碰的指令都发送给服务器时，碰优先于吃。然后服务器会广播此消息。

标准输入（所有）：

```json
{"type":"mpeng","id":2}
```

下面是行牌时的所有可发出的指令：

|*类型* | *指令代码示例*|
|------|-------------|
|出牌 | `{"type":"out","tile":"9M"}`|
|吃  | `{"type":"chi","tiles":["7M","8M"]}`|
|碰  | `{"type":"peng"}`|
|杠  | `{"type":"gang"}`|
|暗杠| `{"type":"agang","tile":"W"}`|
|和  | `{"type":"hu"}`|

### 和牌

当一方有效和牌后，会广播该AI和牌的番数。分数将自动扣除并加到和牌方。

标准输入（所有）：

```json
{"type":"mhu","id":3,"discarder":4,"score":26}
//若discarder与id相等，则为自摸
```

然后即进入下一盘。