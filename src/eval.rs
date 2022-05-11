use crate::{Comparator, Op, Version, VersionReq};

pub(crate) fn matches_req(req: &VersionReq, ver: &Version) -> bool {
    for cmp in &req.comparators {
        if !matches_impl(cmp, ver) {
            return false;
        }
    }

    if ver.pre.is_empty() {
        return true;
    }

    // If a version has a prerelease tag (for example, 1.2.3-alpha.3) then it
    // will only be allowed to satisfy req if at least one comparator with the
    // same major.minor.patch also has a prerelease tag.
    for cmp in &req.comparators {
        if pre_is_compatible(cmp, ver) {
            return true;
        }
    }

    false
}

pub(crate) fn matches_comparator(cmp: &Comparator, ver: &Version) -> bool {
    matches_impl(cmp, ver) && (ver.pre.is_empty() || pre_is_compatible(cmp, ver))
}

fn matches_impl(cmp: &Comparator, ver: &Version) -> bool {
    match cmp.op {
        Op::Exact | Op::Wildcard => matches_exact(cmp, ver),
        Op::Greater => matches_greater(cmp, ver),
        Op::GreaterEq => matches_exact(cmp, ver) || matches_greater(cmp, ver),
        Op::Less => matches_less(cmp, ver),
        Op::LessEq => matches_exact(cmp, ver) || matches_less(cmp, ver),
        Op::Tilde => matches_tilde(cmp, ver),
        Op::Caret => matches_caret(cmp, ver),
        #[cfg(no_non_exhaustive)]
        Op::__NonExhaustive => unreachable!(),
    }
}

fn matches_exact(cmp: &Comparator, ver: &Version) -> bool {
    ver.major == cmp.major
        && cmp.minor.map_or(true, |minor| ver.minor == minor)
        && cmp.patch.map_or(true, |patch| ver.patch == patch)
        && ver.pre == cmp.pre
}

fn matches_greater(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        ver.major > cmp.major
    } else if let Some(minor) = cmp.minor.filter(|&minor| ver.minor != minor) {
        ver.minor > minor
    } else if let Some(patch) = cmp.patch.filter(|&patch| ver.patch != patch) {
        ver.patch > patch
    } else {
        ver.pre > cmp.pre
    }
}

fn matches_less(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        ver.major < cmp.major
    } else if let Some(minor) = cmp.minor.filter(|&minor| ver.minor != minor) {
        ver.minor < minor
    } else if let Some(patch) = cmp.patch.filter(|&patch| ver.patch != patch) {
        ver.patch < patch
    } else {
        ver.pre < cmp.pre
    }
}

fn matches_tilde(cmp: &Comparator, ver: &Version) -> bool {
    if !ver.pre.is_empty() || !cmp.pre.is_empty() {
        matches_exact(cmp, ver)
    } else if ver.major != cmp.major {
        false
    } else if cmp.minor.map_or(false, |minor| ver.minor != minor) {
        false
    } else if let Some(patch) = cmp.patch.filter(|&patch| ver.patch != patch) {
        ver.patch > patch
    } else {
        true
    }
}

fn matches_caret(cmp: &Comparator, ver: &Version) -> bool {
    if !ver.pre.is_empty() || !cmp.pre.is_empty() {
        matches_exact(cmp, ver)
    } else if ver.major != cmp.major {
        false
    } else if let Some(minor) = cmp.minor.filter(|&minor| ver.minor != minor) {
        // if major is 0 than minor is considered as major
        if cmp.major == 0 {
            false
        } else {
            ver.minor > minor
        }
    } else if let Some(patch) = cmp.patch.filter(|&patch| ver.patch != patch) {
        ver.patch > patch
    } else {
        true
    }
}

fn pre_is_compatible(cmp: &Comparator, ver: &Version) -> bool {
    cmp.major == ver.major
        && cmp.minor == Some(ver.minor)
        && cmp.patch == Some(ver.patch)
        && !cmp.pre.is_empty()
}
