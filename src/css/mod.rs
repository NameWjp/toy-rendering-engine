/**
 * 该模块负责 css 的解析，对于 css 文件，有如下格式:
 * [
 *    {
 *      selectors: 描述单个样式块有哪些选择器，
 *      declarations：描述单个样式块有哪些属性（key: value 格式）
 *    },
 *    ...
 * ]
 */
pub mod types;
pub mod parser;