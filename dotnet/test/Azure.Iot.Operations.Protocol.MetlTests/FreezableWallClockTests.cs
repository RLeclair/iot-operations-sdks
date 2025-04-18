﻿// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Azure.Iot.Operations.Protocol.MetlTests
{
    using Xunit;

    public class FreezableWallClockTests
    {
        // Insanely, there seems to be no system API to retrieve this value, so instead we'll assume it based on documentation
        private static readonly TimeSpan AssumedClockResolution = TimeSpan.FromMilliseconds(16);

#pragma warning disable VSTHRD003 // Avoid awaiting foreign Tasks
        public class ClockBehavior
        {
            [Fact]
            public async Task NeverFrozenClockTracksRealTimeAsync()
            {
                FreezableWallClock freezableWallClock = new();

                DateTime lowerBound = DateTime.UtcNow;
                DateTime testTime = freezableWallClock.UtcNow;
                DateTime upperBound = DateTime.UtcNow;
                Assert.True(lowerBound <= testTime);
                Assert.True(testTime <= upperBound);

                await Task.Delay(TimeSpan.FromSeconds(2));

                lowerBound = DateTime.UtcNow;
                testTime = freezableWallClock.UtcNow;
                upperBound = DateTime.UtcNow;
                Assert.True(lowerBound <= testTime);
                Assert.True(testTime <= upperBound);
            }

            [Fact]
            public async Task FrozenClockDoesNotAdvanceAsync()
            {
                TimeSpan delayDuration = TimeSpan.FromSeconds(2);
                FreezableWallClock freezableWallClock = new();

                await freezableWallClock.FreezeTimeAsync();
                DateTime realTimeEarly = DateTime.UtcNow;
                DateTime testTimeEarly = freezableWallClock.UtcNow;

                await Task.Delay(delayDuration);
                DateTime realTimeLate = DateTime.UtcNow;
                DateTime testTimeLate = freezableWallClock.UtcNow;

                Assert.True(realTimeEarly + delayDuration - AssumedClockResolution <= realTimeLate);
                Assert.Equal(testTimeEarly, testTimeLate);
            }

            [Fact]
            public async Task FrozenThenUnfrozenClockMaintainsFixedOffsetAsync()
            {
                TimeSpan delayDuration = TimeSpan.FromSeconds(2);
                FreezableWallClock freezableWallClock = new();

                int ticket = await freezableWallClock.FreezeTimeAsync();
                await Task.Delay(delayDuration);
                await freezableWallClock.UnfreezeTimeAsync(ticket);

                DateTime lowerBound = DateTime.UtcNow;
                DateTime testTime = freezableWallClock.UtcNow;
                DateTime upperBound = DateTime.UtcNow;

                TimeSpan minOffset = lowerBound - testTime;
                TimeSpan maxOffset = upperBound - testTime;

                await Task.Delay(TimeSpan.FromSeconds(2));

                lowerBound = DateTime.UtcNow - maxOffset;
                testTime = freezableWallClock.UtcNow;
                upperBound = DateTime.UtcNow - minOffset;

                Assert.True(lowerBound <= testTime);
                Assert.True(testTime <= upperBound);
            }
        }

        public class Exceptions
        {
            [Fact]
            public async Task UnfreezeUnfrozenClockThrowsExceptionAsync()
            {
                FreezableWallClock freezableWallClock = new();

                await Assert.ThrowsAsync<Exception>(async () => await freezableWallClock.UnfreezeTimeAsync(0));
            }

            [Fact]
            public async Task UnfreezeNonIssuedTicketThrowsExceptionAsync()
            {
                FreezableWallClock freezableWallClock = new();

                int ticket = await freezableWallClock.FreezeTimeAsync();

                await Assert.ThrowsAsync<Exception>(async () => await freezableWallClock.UnfreezeTimeAsync(ticket + 1));
            }

            [Fact]
            public async Task DoubleUnfreezeThrowsExceptionAsync()
            {
                FreezableWallClock freezableWallClock = new();

                int ticket = await freezableWallClock.FreezeTimeAsync();

                await freezableWallClock.UnfreezeTimeAsync(ticket);
                await Assert.ThrowsAsync<Exception>(async () => await freezableWallClock.UnfreezeTimeAsync(ticket + 1));
            }
        }

        public class Matching
        {
            [Fact]
            public async Task MatchedSingularFreezeUnfreezeRestoresAdvancementAsync()
            {
                TimeSpan delayDuration = TimeSpan.FromSeconds(2);
                FreezableWallClock freezableWallClock = new();

                int ticket = await freezableWallClock.FreezeTimeAsync();

                DateTime realTimeEarly = DateTime.UtcNow;
                DateTime testTimeEarly = freezableWallClock.UtcNow;

                await freezableWallClock.UnfreezeTimeAsync(ticket);

                await Task.Delay(delayDuration);
                DateTime realTimeLate = DateTime.UtcNow;
                DateTime testTimeLate = freezableWallClock.UtcNow;

                Assert.True(realTimeEarly + delayDuration - AssumedClockResolution <= realTimeLate);
                Assert.True(testTimeEarly + delayDuration - AssumedClockResolution <= testTimeLate);
            }

            [Fact]
            public async Task MatchedPluralFreezeUnfreezeRestoresAdvancementAsync()
            {
                TimeSpan delayDuration = TimeSpan.FromSeconds(2);
                FreezableWallClock freezableWallClock = new();

                int ticket0 = await freezableWallClock.FreezeTimeAsync();
                int ticket1 = await freezableWallClock.FreezeTimeAsync();

                DateTime realTimeEarly = DateTime.UtcNow;
                DateTime testTimeEarly = freezableWallClock.UtcNow;

                await freezableWallClock.UnfreezeTimeAsync(ticket0);
                await freezableWallClock.UnfreezeTimeAsync(ticket1);

                await Task.Delay(delayDuration);
                DateTime realTimeLate = DateTime.UtcNow;
                DateTime testTimeLate = freezableWallClock.UtcNow;

                Assert.True(realTimeEarly + delayDuration - AssumedClockResolution <= realTimeLate);
                Assert.True(testTimeEarly + delayDuration - AssumedClockResolution <= testTimeLate);
            }

            [Fact]
            public async Task UnmatchedPluralFreezeSingularUnfreezeMaintainsFreezeAsync()
            {
                TimeSpan delayDuration = TimeSpan.FromSeconds(2);
                FreezableWallClock freezableWallClock = new();

                int ticket0 = await freezableWallClock.FreezeTimeAsync();
                int ticket1 = await freezableWallClock.FreezeTimeAsync();

                DateTime realTimeEarly = DateTime.UtcNow;
                DateTime testTimeEarly = freezableWallClock.UtcNow;

                await freezableWallClock.UnfreezeTimeAsync(ticket0);

                await Task.Delay(delayDuration);
                DateTime realTimeLate = DateTime.UtcNow;
                DateTime testTimeLate = freezableWallClock.UtcNow;

                Assert.True(realTimeEarly + delayDuration - AssumedClockResolution <= realTimeLate);
                Assert.Equal(testTimeEarly, testTimeLate);
            }
        }

        public class ClockUnfrozen
        {
            [Theory]
            [InlineData(false)]
            [InlineData(true)]
            public async Task WaitWhenClockUnfrozenAsync(bool relativeWait)
            {
                TimeSpan waitDuration = TimeSpan.FromSeconds(2);
                FreezableWallClock freezableWallClock = new();

                DateTime startTime = freezableWallClock.UtcNow;

                await (relativeWait ? freezableWallClock.WaitForAsync(waitDuration) : freezableWallClock.WaitUntilAsync(startTime + waitDuration));

                Assert.True(startTime + waitDuration - AssumedClockResolution <= freezableWallClock.UtcNow);
            }

            [Theory]
            [InlineData(false)]
            [InlineData(true)]
            public async Task CreateCtsWhenClockUnfrozenAsync(bool relativeWait)
            {
                TimeSpan checkDuration1 = TimeSpan.FromSeconds(5);
                TimeSpan cancelDuration = TimeSpan.FromSeconds(10);
                TimeSpan checkDuration2 = TimeSpan.FromSeconds(15);

                FreezableWallClock freezableWallClock = new();
                DateTime startTime = freezableWallClock.UtcNow;

                CancellationTokenSource cts = freezableWallClock.CreateCancellationTokenSource(cancelDuration);
                Assert.False(cts.IsCancellationRequested);

                await (relativeWait ? freezableWallClock.WaitForAsync(checkDuration1) : freezableWallClock.WaitUntilAsync(startTime + checkDuration1));
                Assert.False(cts.IsCancellationRequested);

                await (relativeWait ? freezableWallClock.WaitForAsync(checkDuration2 - checkDuration1) : freezableWallClock.WaitUntilAsync(startTime + checkDuration2));
                Assert.True(cts.IsCancellationRequested);
            }

            [Theory]
            [InlineData(false)]
            [InlineData(true)]
            public async Task WaitWithCtsWhenClockUnfrozenAsync(bool relativeWait)
            {
                TimeSpan cancelDuration = TimeSpan.FromSeconds(1);
                TimeSpan waitDuration = TimeSpan.FromSeconds(5);
                FreezableWallClock freezableWallClock = new();

                DateTime startTime = freezableWallClock.UtcNow;

                CancellationTokenSource cts = freezableWallClock.CreateCancellationTokenSource(cancelDuration);
                Assert.False(cts.IsCancellationRequested);

                await Assert.ThrowsAsync<TaskCanceledException>(async () => await (relativeWait ? freezableWallClock.WaitForAsync(waitDuration, cts.Token) : freezableWallClock.WaitUntilAsync(startTime + waitDuration, cts.Token)));
                Assert.True(cts.IsCancellationRequested);

                Assert.True(startTime + cancelDuration - AssumedClockResolution <= freezableWallClock.UtcNow);
                Assert.True(freezableWallClock.UtcNow < startTime + waitDuration);
            }

        }

        public class DuringFreeze
        {
            [Theory]
            [InlineData(false)]
            [InlineData(true)]
            public async Task WaitDuringFreezeAsync(bool relativeWait)
            {
                TimeSpan waitDuration = TimeSpan.FromSeconds(3);
                TimeSpan freezeDuration = TimeSpan.FromSeconds(2);
                FreezableWallClock freezableWallClock = new();

                int ticket = await freezableWallClock.FreezeTimeAsync();

                DateTime startTime = freezableWallClock.UtcNow;

                Task waitTask = relativeWait ? freezableWallClock.WaitForAsync(waitDuration) : freezableWallClock.WaitUntilAsync(startTime + waitDuration);
                Assert.False(waitTask.IsCompleted);

                await Task.Delay(freezeDuration);
                await freezableWallClock.UnfreezeTimeAsync(ticket);

                await waitTask;

                Assert.True(startTime + waitDuration - AssumedClockResolution <= freezableWallClock.UtcNow);
            }

            [Theory]
            [InlineData(false)]
            [InlineData(true)]
            public async Task CreateCtsDuringFreezeAsync(bool relativeWait)
            {
                TimeSpan cancelDuration = TimeSpan.FromSeconds(10);
                TimeSpan checkDuration = TimeSpan.FromSeconds(15);
                TimeSpan freezeDuration = TimeSpan.FromSeconds(4);
                FreezableWallClock freezableWallClock = new();

                int ticket = await freezableWallClock.FreezeTimeAsync();

                DateTime startTime = freezableWallClock.UtcNow;

                CancellationTokenSource cts = freezableWallClock.CreateCancellationTokenSource(cancelDuration);
                Assert.False(cts.IsCancellationRequested);

                await Task.Delay(freezeDuration);
                await freezableWallClock.UnfreezeTimeAsync(ticket);
                Assert.False(cts.IsCancellationRequested);

                await (relativeWait ? freezableWallClock.WaitForAsync(checkDuration) : freezableWallClock.WaitUntilAsync(startTime + checkDuration));
                Assert.True(cts.IsCancellationRequested);
            }

            [Theory]
            [InlineData(false)]
            [InlineData(true)]
            public async Task CreateCtsAndWaitDuringFreezeAsync(bool relativeWait)
            {
                TimeSpan cancelDuration = TimeSpan.FromSeconds(1);
                TimeSpan waitDuration = TimeSpan.FromSeconds(5);
                TimeSpan freezeDuration = TimeSpan.FromSeconds(2);

                FreezableWallClock freezableWallClock = new();

                int ticket = await freezableWallClock.FreezeTimeAsync();

                DateTime startTime = freezableWallClock.UtcNow;

                CancellationTokenSource cts = freezableWallClock.CreateCancellationTokenSource(cancelDuration);
                Assert.False(cts.IsCancellationRequested);

                Task waitTask = relativeWait ? freezableWallClock.WaitForAsync(waitDuration, cts.Token) : freezableWallClock.WaitUntilAsync(startTime + waitDuration, cts.Token);
                Assert.False(waitTask.IsCompleted);

                await Task.Delay(freezeDuration);
                await freezableWallClock.UnfreezeTimeAsync(ticket);
                Assert.False(cts.IsCancellationRequested);
                Assert.False(waitTask.IsCompleted);

                await Assert.ThrowsAsync<TaskCanceledException>(async () => await waitTask);
                Assert.True(cts.IsCancellationRequested);

                Assert.True(startTime + cancelDuration - AssumedClockResolution <= freezableWallClock.UtcNow);
                Assert.True(freezableWallClock.UtcNow < startTime + waitDuration);
            }

        }

        public class BeforeFreeze
        {
            [Theory]
            [InlineData(false)]
            [InlineData(true)]
            public async Task WaitBeforeFreezeAsync(bool relativeWait)
            {
                TimeSpan waitDuration = TimeSpan.FromSeconds(3);
                TimeSpan freezeStart = TimeSpan.FromSeconds(1);
                TimeSpan freezeDuration = TimeSpan.FromSeconds(2);
                FreezableWallClock freezableWallClock = new();

                DateTime startTime = freezableWallClock.UtcNow;

                Task waitTask = relativeWait ? freezableWallClock.WaitForAsync(waitDuration) : freezableWallClock.WaitUntilAsync(startTime + waitDuration);
                Assert.False(waitTask.IsCompleted);

                await Task.Delay(freezeStart);
                int ticket = await freezableWallClock.FreezeTimeAsync();

                await Task.Delay(freezeDuration);
                await freezableWallClock.UnfreezeTimeAsync(ticket);

                await waitTask;

                Assert.True(startTime + waitDuration - AssumedClockResolution <= freezableWallClock.UtcNow);
            }

            [Theory]
            [InlineData(false)]
            [InlineData(true)]
            public async Task CreateCtsBeforeFreezeAsync(bool relativeWait)
            {
                TimeSpan cancelDuration = TimeSpan.FromSeconds(10);
                TimeSpan checkDuration = TimeSpan.FromSeconds(15);
                TimeSpan freezeStart = TimeSpan.FromSeconds(2);
                TimeSpan freezeDuration = TimeSpan.FromSeconds(4);

                FreezableWallClock freezableWallClock = new();

                DateTime startTime = freezableWallClock.UtcNow;

                CancellationTokenSource cts = freezableWallClock.CreateCancellationTokenSource(cancelDuration);
                Assert.False(cts.IsCancellationRequested);

                await Task.Delay(freezeStart);
                int ticket = await freezableWallClock.FreezeTimeAsync();
                Assert.False(cts.IsCancellationRequested);

                await Task.Delay(freezeDuration);
                await freezableWallClock.UnfreezeTimeAsync(ticket);
                Assert.False(cts.IsCancellationRequested);

                await (relativeWait ? freezableWallClock.WaitForAsync(checkDuration) : freezableWallClock.WaitUntilAsync(startTime + checkDuration));
                Assert.True(cts.IsCancellationRequested);
            }

            [Theory]
            [InlineData(false)]
            [InlineData(true)]
            public async Task CreateCtsAndWaitBeforeFreezeAsync(bool relativeWait)
            {
                TimeSpan cancelDuration = TimeSpan.FromSeconds(3);
                TimeSpan waitDuration = TimeSpan.FromSeconds(10);
                TimeSpan freezeStart = TimeSpan.FromSeconds(1);
                TimeSpan freezeDuration = TimeSpan.FromSeconds(2);

                FreezableWallClock freezableWallClock = new();

                DateTime startTime = freezableWallClock.UtcNow;

                CancellationTokenSource cts = freezableWallClock.CreateCancellationTokenSource(cancelDuration);
                Assert.False(cts.IsCancellationRequested);

                Task waitTask = relativeWait ? freezableWallClock.WaitForAsync(waitDuration, cts.Token) : freezableWallClock.WaitUntilAsync(startTime + waitDuration, cts.Token);
                Assert.False(waitTask.IsCompleted);

                await Task.Delay(freezeStart);
                int ticket = await freezableWallClock.FreezeTimeAsync();
                Assert.False(cts.IsCancellationRequested);
                Assert.False(waitTask.IsCompleted);

                await Task.Delay(freezeDuration);
                await freezableWallClock.UnfreezeTimeAsync(ticket);
                Assert.False(cts.IsCancellationRequested);
                Assert.False(waitTask.IsCompleted);

                await Assert.ThrowsAsync<TaskCanceledException>(async () => await waitTask);
                Assert.True(cts.IsCancellationRequested);

                Assert.True(startTime + cancelDuration - AssumedClockResolution <= freezableWallClock.UtcNow);
                Assert.True(freezableWallClock.UtcNow < startTime + waitDuration);
            }
        }

        public class BeforeAndDuringFreeze
        {
            [Theory(Skip = "flaky")]
            [InlineData(false)]
            [InlineData(true)]
            public async Task CreateCtsBeforeAndWaitDuringFreezeAsync(bool relativeWait)
            {
                TimeSpan cancelDuration = TimeSpan.FromSeconds(3);
                TimeSpan waitDuration = TimeSpan.FromSeconds(10);
                TimeSpan freezeStart = TimeSpan.FromSeconds(1);
                TimeSpan freezeDuration = TimeSpan.FromSeconds(2);

                FreezableWallClock freezableWallClock = new();

                DateTime startTime = freezableWallClock.UtcNow;

                CancellationTokenSource cts = freezableWallClock.CreateCancellationTokenSource(cancelDuration);
                Assert.False(cts.IsCancellationRequested);

                await Task.Delay(freezeStart);
                int ticket = await freezableWallClock.FreezeTimeAsync();
                Assert.False(cts.IsCancellationRequested);

                Task waitTask = relativeWait ? freezableWallClock.WaitForAsync(waitDuration, cts.Token) : freezableWallClock.WaitUntilAsync(startTime + waitDuration, cts.Token);
                Assert.False(waitTask.IsCompleted);

                await Task.Delay(freezeDuration);
                await freezableWallClock.UnfreezeTimeAsync(ticket);
                Assert.False(cts.IsCancellationRequested);
                Assert.False(waitTask.IsCompleted);

                await Assert.ThrowsAsync<TaskCanceledException>(async () => await waitTask);
                Assert.True(cts.IsCancellationRequested);

                Assert.True(startTime + cancelDuration - AssumedClockResolution <= freezableWallClock.UtcNow);
                Assert.True(freezableWallClock.UtcNow < startTime + waitDuration);
            }

        }

        public class WaitAsync
        {
            [Fact]
            public async Task WaitAsyncCompletesNormallyAsync()
            {
                FreezableWallClock freezableWallClock = new();
                TaskCompletionSource<int> tcs = new();
                CancellationTokenSource cts = new();

                Task waitTask = freezableWallClock.WaitAsync(tcs.Task, TimeSpan.FromMinutes(1), cts.Token);

                await Task.Delay(TimeSpan.FromSeconds(1));
                tcs.SetResult(3);
                await waitTask;

                Assert.True(tcs.Task.IsCompleted);
                Assert.False(tcs.Task.IsFaulted);

                int result = await tcs.Task;
                Assert.Equal(3, result);
            }

            [Fact]
            public async Task WaitAsyncTimesOutAsync()
            {
                FreezableWallClock freezableWallClock = new();
                TaskCompletionSource<int> tcs = new();
                CancellationTokenSource cts = new();

                await Assert.ThrowsAsync<TimeoutException>(async () => { await freezableWallClock.WaitAsync(tcs.Task, TimeSpan.FromSeconds(1), cts.Token); });
            }

            [Fact]
            public async Task WaitAsyncCanceledAsync()
            {
                FreezableWallClock freezableWallClock = new();
                TaskCompletionSource<int> tcs = new();
                CancellationTokenSource cts = new();

                Task waitTask = freezableWallClock.WaitAsync(tcs.Task, TimeSpan.FromMinutes(1), cts.Token);

                await Task.Delay(TimeSpan.FromSeconds(1));
                await cts.CancelAsync();

                await Assert.ThrowsAsync<OperationCanceledException>(async () => { await waitTask; });
            }
#pragma warning restore VSTHRD003 // Avoid awaiting foreign Tasks
        }
    }
}
