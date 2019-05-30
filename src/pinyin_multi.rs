use crate::data::{HETERONYM_TABLE, PINYIN_DATA};
use crate::{get_block_and_index, Pinyin, PinyinData};
use std::convert::TryFrom;
use std::str::Chars;

/// 单个字符的多音字信息
///
/// *仅在启用 `heteronym` 特性时可用*
#[derive(Copy, Clone)]
pub struct PinyinMulti {
    first: &'static PinyinData,
    other_indexes: &'static [u16],
}

impl PinyinMulti {
    /// 对应字符不同发音的数量
    pub fn count(self) -> usize {
        self.other_indexes.len() + 1
    }

    /// 获取指定序号的拼音，如果序号超过总数则panic
    pub fn get(self, idx: usize) -> Pinyin {
        self.get_opt(idx).unwrap()
    }

    /// 获取指定序号的拼音，如果序号超过总数则返回 `None`
    pub fn get_opt(self, idx: usize) -> Option<Pinyin> {
        if idx == 0 {
            Some(Pinyin(self.first))
        } else {
            self.other_indexes
                .get(usize::try_from(idx).unwrap() - 1)
                .map(|i| Pinyin(&PINYIN_DATA[usize::try_from(*i).unwrap()]))
        }
    }
}

impl IntoIterator for PinyinMulti {
    type Item = Pinyin;
    type IntoIter = PinyinMultiIter;

    fn into_iter(self) -> PinyinMultiIter {
        PinyinMultiIter {
            inner: self,
            index: 0,
        }
    }
}

/// *辅助迭代器*，用于迭代一个字的多个拼音
pub struct PinyinMultiIter {
    inner: PinyinMulti,
    index: usize,
}

impl Iterator for PinyinMultiIter {
    type Item = Pinyin;

    fn next(&mut self) -> Option<Pinyin> {
        self.inner.get_opt(self.index).map(|pinyin| {
            self.index += 1;
            pinyin
        })
    }
}

/// 用于获取多音字信息的trait
///
/// *仅在启用 `heteronym` 特性时可用*
pub trait ToPinyinMulti {
    type Output;
    fn to_pinyin_multi(&self) -> Self::Output;
}

/// ```
/// # #[cfg(feature = "with_tone")] {
/// use pinyin::{Pinyin, ToPinyinMulti};
/// let mut iter = '还'.to_pinyin_multi().unwrap().into_iter();
/// let mut next_pinyin = || iter.next().map(Pinyin::with_tone);
/// assert_eq!(next_pinyin(), Some("hái"));
/// assert_eq!(next_pinyin(), Some("fú"));
/// assert_eq!(next_pinyin(), Some("huán"));
/// assert_eq!(next_pinyin(), None);
/// # }
/// ```
impl ToPinyinMulti for char {
    type Output = Option<PinyinMulti>;

    fn to_pinyin_multi(&self) -> Option<PinyinMulti> {
        get_block_and_index(*self).and_then(|(block, index)| {
            let first = match usize::try_from(block.data[index]).unwrap() {
                0 => return None,
                idx => &PINYIN_DATA[idx],
            };
            let idx = usize::try_from(block.heteronym[index]).unwrap();
            let other_indexes = HETERONYM_TABLE[idx];
            Some(PinyinMulti {
                first,
                other_indexes,
            })
        })
    }
}

/// ```
/// # #[cfg(feature = "with_tone")] {
/// use pinyin::{Pinyin, ToPinyinMulti};
/// let mut iter = "还没".to_pinyin_multi();
/// let mut next_heteronym = || {
///     iter.next()
///         .and_then(|m| m)
///         .map(|m| m.into_iter().map(Pinyin::with_tone).collect::<Vec<_>>())
/// };
/// assert_eq!(next_heteronym(), Some(vec!["hái", "fú", "huán"]));
/// assert_eq!(next_heteronym(), Some(vec!["méi", "mò", "me"]));
/// assert_eq!(next_heteronym(), None);
/// # }
/// ```
impl<'a> ToPinyinMulti for &'a str {
    type Output = PinyinMultiStrIter<'a>;

    #[inline]
    fn to_pinyin_multi(&self) -> Self::Output {
        PinyinMultiStrIter(self.chars())
    }
}

/// *辅助迭代器*，用于获取字符串的多音字信息
pub struct PinyinMultiStrIter<'a>(Chars<'a>);

impl<'a> Iterator for PinyinMultiStrIter<'a> {
    type Item = Option<PinyinMulti>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| c.to_pinyin_multi())
    }
}
