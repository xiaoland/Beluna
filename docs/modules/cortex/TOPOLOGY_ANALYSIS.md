# Cortex 模块拓扑与时序分析

## 概述

Cortex 是一个**无状态的认知边界**，负责：

- 消费：感知数据 + 物理状态 + 认知状态
- 产出：动作列表 + 新认知状态 + 等待标志

核心架构：**输入处理 → Primary 推理 → 输出处理 → 补丁应用**

---

## 架构核心要素

### 1. 数据流三阶段

#### 第一阶段：并行输入处理（Input Helpers）

4个 helpers 并行运行，各自独立处理一个认知器官：

| Helper | 输入 | 输出 | 职责 |
|--------|------|------|------|
| **Sense** | Sense[] | `<somatic-senses>` | 组装感知、缓存、Postman信封 |
| **ActDescriptor** | act descriptors | `<somatic-act-descriptor-catalog>` | 汇总动作、生成fq_act_id |
| **GoalTree** | goal_tree | `<instincts>` + `<willpower-matrix>` | 本能转换、目标树JSON展开 |
| **L1Memory** | l1_memory | `<focal-awareness>` | 短期记忆格式化 |

**约束：**

- Sense passthrough: ≤ 2,048 字节直接通过
- L1Memory 上限: 10 条
- Goal tree 缓存键: MD5(user_partition)

#### 第二阶段：Primary 认知微循环（LLM）

- **输入：** Input IR XML 完整结构
- **处理：**
  - 最多 4 轮内部迭代（max_internal_steps）
  - 可调用工具：`expand-sense-raw` 或 `expand-sense-with-sub-agent`
  - 反馈循环支持
- **输出：** 包含 4 个部分的 XML 文本
- **超时：** 可配置 deadline

#### 第三阶段：并行输出处理（Output Helpers）

3个 helpers 并行运行，解析 Primary 输出：

| Helper | 输入 | 输出 | 职责 |
|--------|------|------|------|
| **Acts** | `<somatic-acts>` | Act[] | 解析动作、生成act_instance_id |
| **GoalTreePatch** | `<willpower-matrix-patch>` | GoalTreePatchOp[] | 解析Sprout/Prune/Tilt操作 |
| **L1MemoryFlush** | `<new-focal-awareness>` | L1Memory | 焦点意识更新 |

---

## Input IR 完整结构

```xml
<input-ir>
  <somatic-senses>
    <!-- sense XML 元素，带 sense_instance_id -->
  </somatic-senses>
  <somatic-act-descriptor-catalog>
    <!-- 所有可用动作及其完整描述 -->
  </somatic-act-descriptor-catalog>
  <instincts>
    <!-- 4 条不可变的本能规则 -->
  </instincts>
  <willpower-matrix>
    <!-- 用户定义的动态目标树 -->
  </willpower-matrix>
  <focal-awareness>
    <!-- 过去的自我留下的记忆与推论 -->
  </focal-awareness>
</input-ir>
```

---

## Output IR 完整结构

```xml
<output-ir>
  <somatic-acts>
    <!-- 决定采取的物理或认知动作 -->
  </somatic-acts>
  <willpower-matrix-patch>
    <!-- 对目标树的修改操作 -->
  </willpower-matrix-patch>
  <new-focal-awareness>
    <!-- 新的焦点意识，为下一个自我准备 -->
  </new-focal-awareness>
  <is-wait-for-sense>true/false</is-wait-for-sense>
</output-ir>
```

---

## 关键数据结构

### CognitionState

```rust
pub struct CognitionState {
    pub revision: u64,
    pub goal_tree: GoalTree,
    pub l1_memory: L1Memory,
}

pub struct GoalTree {
    pub root_partition: Vec<String>,  // 4 条不变本能
    pub user_partition: Vec<GoalNode>, // 用户自定义目标
}

pub type L1Memory = Vec<String>;  // 短期记忆，最多10条
```

### ReactionLimits 配置限制

```rust
pub struct ReactionLimits {
    pub max_cycle_time_ms: u64,          // 总轮询超时
    pub max_internal_steps: u8,          // Primary 迭代上限（默认4）
    pub sense_passthrough_max_bytes: usize, // 直通大小（默认2KB）
    pub max_l1_memory_entries: usize,    // L1 记忆上限（默认10）
    pub max_primary_calls: u8,           // Primary 调用数（默认1）
    pub max_sub_calls: u8,               // Sub-agent 调用数（默认2）
    // ... 其他限制
}
```

---

## Primary Engine 认知微循环

### 执行流程

1. **第1轮迭代**
   - 分析 Input IR（instincts、willpower-matrix、focal-awareness）
   - 解释感知数据（somatic-senses）
   - 交叉验证感知 vs 记忆

2. **迭代决策**
   - 是否需要感知扩展？
   - 调用工具 → 反馈 → 重新推理

3. **最多 max_internal_steps 轮**
   - 每轮可能触发工具调用
   - 反馈循环优化决策

4. **最终输出**
   - 确定 `<somatic-acts>`
   - 确定 `<willpower-matrix-patch>`
   - 确定 `<new-focal-awareness>`
   - 确定 `<is-wait-for-sense>`

---

## Helpers 缓存机制

### 仅适用于输入 Helpers

- **Sense cache**：MD5(原始感知描述符) → 上次处理结果
- **ActDescriptor cache**：MD5(描述符集) → 动作目录 XML
- **GoalTree cache**：MD5(user_partition) → 转换后的 willpower-matrix

**约束：**

- 进程作用域（内存）
- 无持久化
- 无跨请求共享

---

## 容错与恢复

### 三层容错策略

#### 策略 A：输入失败 → 降级替代

```
Sense Helper 失败 → 绕过 LLM → 确定性输出
ActDescriptor Helper 失败 → 空目录 → 继续
GoalTree Helper 失败 → 默认一次性追求 → 继续
L1Memory Helper 失败 → 用 bullet 列表 → 继续
```

#### 策略 B：Primary 失败 → Noop 回退

```
Primary 超时 → fail-closed noop → 保留原认知状态
Primary LLM 错误 → fail-closed noop → 保留原认知状态
Output IR 契约失败 → fail-closed noop → 保留原认知状态
```

#### 策略 C：输出失败 → 跳过部分输出

```
Acts Output Helper 失败 → 空动作列表
GoalTree Patch 失败 → 零操作
L1Memory Flush 失败 → 保留旧状态
↓
应用剩余补丁 → 继续
```

### 错误处理流程

```
输入阶段 ─── 部分失败 ─── 使用替代 ──┐
             │ 全部成功        │
             └────────────────┤
                              ↓
Primary 阶段 ─── 超时/失败 ─── Noop 回退 ──┐
             │ 成功              │
             └───────────────────┤
                                 ↓
输出阶段 ─── 部分失败 ─── 跳过+继续 ──┐
             │ 全部成功        │
             └────────────────┤
                              ↓
补丁应用 ─── 合并状态 ──┐
             │ 失败      └─→ Noop 回退
             └────────────→ 成功 ──→ 返回结果
```

---

## 遥测与监控

### CortexTelemetryEvent 类型

```rust
pub enum CortexTelemetryEvent {
    ReactionStarted { cycle_id: u64 },
    StageFailed { cycle_id: u64, stage: &'static str },
    ReactionCompleted { cycle_id: u64, act_count: usize },
    NoopFallback { cycle_id: u64, reason: &'static str },
}
```

**监控点：**

- 轮询启动 → 记录 cycle_id
- 任何阶段失败 → 记录失败阶段名
- 降级回退 → 记录原因
- 轮询完成 → 记录生成的动作数

---

## 关键约束与不变量

### 无状态性

- Cortex 本身不持久化认知或目标状态
- 所有状态通过参数传入，修改后返回

### IR 隔离

- Input IR 和 Output IR 是 Rust 所有的内部信封
- 不暴露给 Primary LLM 的系统提示
- 不暴露传输 ID（如 sense_instance_id）

### 原子性

- 每个轮询（cognition cycle）是相对独立的单元
- 无中间结果的跨周期备份

### 确定性降级

- Helpers 失败时，使用纯 Rust 的确定性替代
- 不触发级联失败

### 超时保护

- 所有异步操作受 deadline 约束
- deadline = max_cycle_time_ms
- Primary 超时 → fail-closed noop

### 认知主权

- Primary 是唯一的认知决策器
- Helpers 是认知器官，辅助但不决策

---

## 实现要点

### 并行性

- 输入 Helpers：4 个并行 (tokio::join!)
- 输出 Helpers：3 个并行 (tokio::join!)
- 充分利用异步 I/O

### 内存管理

- 缓存仅在进程内，无持久化
- 大感知数据通过 Postman 信封压缩
- L1 记忆溢出检测与警告

### 日志与调试

- 每个阶段的输入输出记录
- cycle_id 作为关联键
- tracing 框架用于分布式追踪

---

## 文件组织

```
core/src/cortex/
├── mod.rs                              # 导出公共接口
├── runtime.rs                          # Cortex 运行时（1032 行）
├── cognition.rs                        # 认知状态定义
├── cognition_patch.rs                  # 补丁应用逻辑
├── ir.rs                               # Input/Output IR 构建与解析
├── prompts.rs                          # 系统提示与用户提示
├── clamp.rs                            # 边界检查
├── error.rs                            # 错误定义
├── types.rs                            # 数据结构（CortexOutput 等）
├── helpers/
│   ├── mod.rs                          # Helper 抽象与工具定义
│   ├── sense_input_helper.rs          # 感知处理
│   ├── act_descriptor_input_helper.rs # 动作描述符处理
│   ├── goal_tree_input_helper.rs      # 目标树处理
│   ├── l1_memory_input_helper.rs      # 短期记忆处理
│   ├── acts_output_helper.rs          # 动作提取
│   ├── goal_tree_patch_output_helper.rs # 目标树补丁提取
│   └── l1_memory_flush_output_helper.rs # 焦点意识更新
├── testing.rs                          # 测试钩子
└── AGENTS.md                           # 这个文档
```

---

## 总结

Cortex 是一个精心设计的认知处理边界，通过：

1. **分层设计** 分离关注点
2. **IR-Based 通信** 确保结构化、可验证的数据流
3. **并行处理** 最大化吞吐
4. **降级策略** 保证容错性
5. **工具支持** 赋予 LLM 认知自适应能力

使得信息（数据）、意图（目标树）、记忆（焦点意识）、能力（动作）能够在每个轮询周期内有序地流动，形成认知的自我更新与演进。
