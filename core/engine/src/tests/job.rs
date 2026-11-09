use std::{
    cell::{Cell, RefCell},
    pin::pin,
    rc::Rc,
};

use futures_lite::future;

use crate::{
    Context, JsNativeError, JsResult, JsValue, TestAction,
    context::{ContextBuilder, time::FixedClock},
    job::{GenericJob, JobExecutor, NativeAsyncJob, SimpleJobExecutor},
    run_test_actions_with,
};

#[test]
fn test_async_job_not_blocking_event_loop() {
    let clock = Rc::new(FixedClock::default());
    let context = &mut ContextBuilder::default()
        .clock(clock.clone())
        .build()
        .unwrap();

    run_test_actions_with(
        [TestAction::inspect_context_async(async move |ctx| {
            let executor = ctx.downcast_job_executor::<SimpleJobExecutor>().unwrap();
            let ctx = &RefCell::new(ctx);

            let mut event_loop = pin!(future::poll_once(executor.run_jobs_async(ctx)));

            // There are no jobs in our queue. Push
            // an async job that will consistently yield to the executor.
            ctx.borrow_mut().enqueue_job(
                NativeAsyncJob::new(async |_| {
                    loop {
                        future::yield_now().await;
                    }
                })
                .into(),
            );

            // Then, start the event loop
            assert!(event_loop.as_mut().await.is_none());

            let checker = Rc::new(Cell::new(false));
            {
                let checker = checker.clone();
                // At this point, the event loop should have yielded again to the async executor.
                // Thus, enqueue a generic job that should resolve in the next loop.
                let realm = ctx.borrow().realm().clone();
                ctx.borrow_mut().enqueue_job(
                    GenericJob::new(
                        move |_| {
                            checker.set(true);
                            Ok(JsValue::undefined())
                        },
                        realm,
                    )
                    .into(),
                );
            }

            // Next iteration of the event loop
            assert!(event_loop.as_mut().await.is_none());

            // At this point, our generic job should have been executed.
            assert!(checker.get());
        })],
        context,
    );
}

/// Queue `ok`, a failing job, then `ok` again, run the queue, and report how many `ok` jobs ran
/// and whether `run_jobs` returned the failure.
fn run_around_a_failing_job(executor: SimpleJobExecutor) -> (u32, bool) {
    let ran = Rc::new(Cell::new(0u32));
    let mut context = ContextBuilder::default()
        .job_executor(Rc::new(executor))
        .build()
        .unwrap();
    let realm = context.realm().clone();
    for step in 0..3 {
        let ran = ran.clone();
        let job = GenericJob::new(
            move |_: &mut Context| -> JsResult<JsValue> {
                if step == 1 {
                    return Err(JsNativeError::typ().with_message("boom").into());
                }
                ran.set(ran.get() + 1);
                Ok(JsValue::undefined())
            },
            realm.clone(),
        );
        context.enqueue_job(job.into());
    }
    let failed = context.run_jobs().is_err();
    (ran.get(), failed)
}

#[test]
fn job_error_clears_the_queue_by_default() {
    // The default stays "bail on the first error": the job queued after the failure is lost.
    assert_eq!(
        run_around_a_failing_job(SimpleJobExecutor::new()),
        (1, true)
    );
}

#[test]
fn continue_on_error_keeps_the_rest_of_the_queue_and_returns_the_first_error() {
    let executor = SimpleJobExecutor::new().with_continue_on_error(true);
    assert_eq!(run_around_a_failing_job(executor), (2, true));
}
