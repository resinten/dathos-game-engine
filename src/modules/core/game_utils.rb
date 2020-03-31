# frozen_string_literal: true

# Base game utilities
module GameUtils
  def self.make_coroutine(block)
    Enumerator.new do |yielder|
      wait = Waiter.new(yielder)
      block.call(wait)
      wait.done
    end
  end

  def self.make_run_for(duration, block)
    Enumerator.new do |yielder|
      wait = Waiter.new(yielder)
      started = Game.time
      loop do
        time_since = Game.time_since started
        block.call(time_since, duration)
        time_since > duration ? wait.done : wait.next_frame
      end
      wait.done
    end
  end
end
