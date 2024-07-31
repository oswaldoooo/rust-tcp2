// #![feature(test)]
mod sasync {
    use std::{collections::VecDeque, sync::Arc};
    struct Task {
        f: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
        hand: Handle,
    }
    #[derive(Default, Clone)]
    pub struct Handle {
        l: Arc<std::sync::Mutex<bool>>,
        n: Arc<std::sync::Condvar>,
        _closed: Arc<std::sync::RwLock<bool>>,
    }
    static mut tasks: std::sync::Mutex<VecDeque<Task>> = std::sync::Mutex::new(VecDeque::new());
    static mut __task_lock: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
    static mut __task_cond: std::sync::Condvar = std::sync::Condvar::new();
    #[allow(dead_code)]
    pub fn spawn<F>(f: F) -> Handle
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let hand = Handle::default();
        
        unsafe {
            // println!("wait lock");
            let mut ll=tasks.lock().unwrap();
            ll.push_back(Task {
                f: Box::pin(f),
                hand: hand.clone(),
            });
            // println!("push success");
            __task_cond.notify_one();
            drop(ll);
        };
        
        return hand;
    }
    impl Handle {
        pub fn join(&mut self) {
            // println!("read lock");
            if *self._closed.read().unwrap() {
                return;
            }
            let ll = self.l.lock().unwrap();
            drop(self.n.wait(ll));
        }
    }
    pub fn start() {
        // use std::task::Wake;
        let w = futures::task::noop_waker();
        let mut ctx = std::task::Context::from_waker(&w);
        unsafe {
            loop {
                let mut ll=tasks.lock().unwrap();
                if let Some(mut e) = ll.pop_front() {
                    
                    drop(ll);
                    // println!("into");
                    if e.f.as_mut().poll(&mut ctx).is_pending() {
                        // println!("zone3 lock!");
                        tasks.lock().unwrap().push_back(e);
                    } else {
                        // println!("wait write lock");
                        *e.hand._closed.write().unwrap() = true;
                        e.hand.n.notify_all();
                    }
                } else {
                    break;
                    // drop(ll);
                    // let ll = __task_lock.lock().unwrap();
                    // drop(__task_cond.wait_timeout(ll, std::time::Duration::from_secs(2)).unwrap());
                }
            }
        }
    }
    #[derive(Default)]
    struct Waker {}
    impl std::task::Wake for Waker {
        fn wake(self: std::sync::Arc<Self>) {}
    }
}
#[cfg(test)]
mod async_test {
    use crate::sasync::{self, start};
    #[test]
    fn test_std() {
        let mut h = super::sasync::spawn(async {
            println!("hello");
            async { println!("im async method") }.await;
            let mut hlist = Vec::new();
            for i in 0..5 {
                println!("do task {i}");
                hlist.push(sasync::spawn(async move{ println!("do {i}") }));
            }
            let mut e = 0;
            for i in hlist.iter_mut() {
                println!("wait task {e}");
                e += 1;
                // i.join();
            }
        });
        let hand = std::thread::spawn(|| {
            start();
        });
        //main
        
        hand.join();
    }
    #[test_lib::test]
    fn test_test(){
        println!("im body");
        1+1;
    }
    
    // extern crate test;
    // #[bench]
    // fn ben_drop(b:&mut test::Bencher){

    // }
}
