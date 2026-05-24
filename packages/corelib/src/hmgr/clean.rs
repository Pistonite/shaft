use crate::hmgr;

pub fn clean_home() {
    let bar = cu::progress("cleaning shaft core").spawn();
    {
        let bar = bar.child("cleaning install-old dir").keep(true).spawn();
        if let Err(e) = cu::fs::rec_remove(hmgr::paths::install_old_root()) {
            cu::warn!("failed to remove install-old dir: {e:?}");
        }
        bar.done();
    }
    {
        let bar = bar.child("cleaning download dir").keep(true).spawn();
        if let Err(e) = cu::fs::rec_remove(hmgr::paths::download_root()) {
            cu::warn!("failed to remove download dir: {e:?}");
        }
        bar.done();
    }
    {
        let bar = bar.child("cleaning temp dir").keep(true).spawn();
        if let Err(e) = cu::fs::rec_remove(hmgr::paths::temp_root()) {
            cu::warn!("failed to remove temp dir: {e:?}");
        }
        bar.done();
    }
    {
        let bar = bar.child("cleaning repo dir").keep(true).spawn();
        if let Err(e) = cu::fs::rec_remove(hmgr::paths::repo()) {
            cu::warn!("failed to remove repo dir: {e:?}");
        }
        bar.done();
    }
    bar.done();
}
