#!/usr/bin/env ruby
# typed: true
# frozen_string_literal: true
require "bundler/setup"
require "optparse"
require "HDRHistogram"
require "fileutils"
require "lmdb"
require "active_record"
require "snappy"
require "sorbet-runtime"
require "dalli"
require "paquito"

class EphemeralCacheClient
    extend T::Sig

    sig { params(options: T.nilable(Hash)).void }
    def initialize(options = nil)
        options = options ? options.dup : {}

        @memcached_servers = options[:memcached_servers].nil? ? ENV["MEMCACHE_SERVERS"] : options[:memcached_servers]
        @coder = Paquito::SingleBytePrefixVersionWithStringBypass.new(
        0x00,
        {
            0x00 =>
            Paquito::CodecFactory.build([
                Symbol,
                Time,
                DateTime,
                Date,
                BigDecimal,
                ActiveRecord::Base,
                ActiveSupport::HashWithIndifferentAccess,
                ActiveSupport::TimeWithZone,
                Set,
            ]),
        },
        )
    end

    sig do
        params(
        key: String,
        value: T.untyped,
        ttl: T.nilable(Integer),
        options: T.nilable(T::Hash[Symbol, T.untyped]),
        ).returns(T.untyped)
    end
    def set(key, value, ttl = nil, options = nil)
        options = options ? options.dup : {}
        client.set(key, value, ttl, options)
    end

    sig { params(key: String).returns(T.untyped) }
    def get(key)
        client.get(key, false)
    end

    sig { params(keys: T::Array[String]).returns(T::Hash[String, T.untyped]) }
    def get_multi(keys)
        client.get_multi(keys)
    end

    sig do
        params(
        keys: T::Hash[String, T.untyped],
        ttl: T.nilable(Integer),
        options: T.nilable(T::Hash[Symbol, T.untyped]),
        ).returns(T::Hash[String, T.untyped])
    end
    def set_multi(keys, ttl = nil, options = nil)
        options = options ? options.dup : {}
        client.multi do
        keys.each do |key, value|
            client.set(key, value, ttl, options)
        end
        end
    end

    sig { params(key: String).returns(T::Boolean) }
    def delete(key)
        !!client.delete(key)
    end

    sig { params(key: String, amount: T.nilable(Integer)).returns(T.nilable(Integer)) }
    def incr(key, amount = 1)
        client.incr(key, amount)
    end

    sig { params(key: String, amount: T.nilable(Integer)).returns(T.nilable(Integer)) }
    def decr(key, amount = 1)
        client.decr(key, amount)
    end

    private

    def client
        @memcached_client ||=
        Dalli::Client.new(
            @memcached_servers,
            protocol: :binary,
            compress: false,
            threadsafe: false,
            serializer: @coder,
        )
    end
end

class LocalLMDB
  def initialize(folder, mapsize: 2.gigabyte, nometasync: true, nosync: true)
    FileUtils.rm_rf(folder)
    FileUtils.mkdir_p(folder)
    @env = LMDB.new(folder, mapsize: mapsize, nometasync: nometasync, nosync: nosync)
    @db = db
  end

  def db
    @db ||= @env.database("db", create: true)
  end

  def transaction
    @env.transaction do |txn|
      yield txn
    end
  end

  def get(key)
    transaction do
      @db.get(key)
    end
  end

  def set(key, value)
    transaction do
      @db.put(key, value)
    end
  end

  def delete(key)
    transaction do
      @db.delete(key)
    end
    rescue LMDB::Error::NOTFOUND
    nil
  end
end

LATENCY_HDR_MIN_VALUE = 10
LATENCY_HDR_MAX_VALUE = 60000000
LATENCY_HDR_SIGDIGTS = 3

LMDB_FOLDER = "./data/lmdb"

def sanity_check(client, payload)
  client.delete("foo")
  client.set("foo", payload)
  roundtrip = client.get("foo")
  if roundtrip != payload
    puts "Client: #{client.inspect}"
    puts "Payload: #{payload.inspect}"
    puts "Rountrip: #{roundtrip.inspect}"
    raise "Fail!"
  end
end

def measure
  start = Process.clock_gettime(Process::CLOCK_MONOTONIC, :microsecond)
  yield
  Process.clock_gettime(Process::CLOCK_MONOTONIC, :microsecond) - start
end

def hdr_results(hdr, filename)
  # Create Array and its header
  response_array = []
  response_array << "#{"Value".rjust(12)} \
    #{"Percentile".rjust(14)} \
    #{"TotalCount".rjust(10)} \
    #{"1/(1-Percentile)".rjust(14)}\n\n"

  # Create the rest of the rows
  # rubocop:disable Layout/MultilineArrayLineBreaks
  quantiles = [
    0.000000,0.050000,0.100000,0.150000,0.200000,0.250000,0.300000,0.350000,0.400000,0.450000,0.500000,0.525000,0.550000,0.575000,0.600000,0.625000,0.650000,0.675000,0.700000,0.725000,0.750000,0.762500,0.775000,0.787500,0.800000,0.812500,0.825000,0.837500,0.850000,0.862500,0.875000,0.881250,0.887500,0.893750,0.900000,0.906250,0.912500,0.918750,0.925000,0.931250,0.937500,0.940625,0.943750,0.946875,0.950000,0.953125,0.956250,0.959375,0.962500,0.965625,0.968750,0.970313,0.971875,0.973437,0.975000,0.976562,0.978125,0.979688,0.981250,0.982812,0.984375,0.985156,0.985938,0.986719,0.987500,0.988281,0.989062,0.989844,0.990625,0.991406,0.992188,0.992578,0.992969,0.993359,0.993750,0.994141,0.994531,0.994922,0.995313,0.995703,0.996094,0.996289,0.996484,0.996680,0.996875,0.997070,0.997266,0.997461,0.997656,0.997852,0.998047,0.998145,0.998242,0.998340,0.998437,0.998535,0.998633,0.998730,0.998828,0.998926,0.999023,0.999072,0.999121,0.999170,0.999219,0.999268,0.999316,0.999365,0.999414,0.999463,0.999512,0.999536,0.999561,0.999585,0.999609,0.999634,0.999658,0.999683,0.999707,0.999731,0.999756,0.999768,0.999780,0.999792,0.999805,0.999817,0.999829,0.999841,0.999854,0.999866,0.999878,0.999884,0.999890,1.000000
  ]
  # rubocop:enable Layout/MultilineArrayLineBreaks

  quantiles.sort.each do |q|
    p_value_raw = hdr.percentile(q*100) / 1000.0
    p_value = format("%.3f", p_value_raw).to_s.rjust(12)
    p_percentile = format("%.6f", q).to_s.rjust(14)
    p_totalcount = (hdr.count { |n| n <= p_value_raw }).to_s.rjust(10)
    p_invert_percentile = format("%.2f", (1.0 / (1.0 - q))).to_s.rjust(14)
    response_array << "#{p_value} #{p_percentile} #{p_totalcount} #{p_invert_percentile}\n"
  end

  # add footer
  response_array << "#[Mean        = #{format("%.3f", (hdr.mean / 1000.0)).to_s.rjust(12)},\
    StdDeviation   = #{format("%.3f", (hdr.stddev / 1000.0)).to_s.rjust(12)}]\n"
  response_array << "#[Max         = #{format("%.3f", (hdr.max / 1000.0)).to_s.rjust(12)},\
    Total count    = #{hdr.count.to_s.rjust(12)}]\n"
  response_array << "#[Buckets     = #{13.to_s.rjust(12)},    SubBuckets     = #{2048.to_s.rjust(12)}]\n"

  response_array.join

  File.open(filename, "w") do |file|
    response_array.each { |line| file.write(line) }
  end
end

def bench(options)
  times = {}
  client = EphemeralCacheClient.new(
    {
      memcached_servers: [options[:socket] || "#{options[:server]}:#{options[:port]}"],
    },
  )
  if options[:type] == "lmdb" 
    client = LocalLMDB.new(LMDB_FOLDER)
  end

  sanity_check(client, options[:payload])

  options[:requests].times do
    if rand < options[:ratio]
      times["SET"] = times.fetch("SET", []).push(measure do
        client.send("set", "foo", options[:payload])
      end)
    else
      times["GET"] = times.fetch("GET", []).push(measure do
        client.send("get", "foo")
      end)
    end
  end

  times.transform_values do |times|
    r = {
      ops: times.length,
      histogram: HDRHistogram.new(LATENCY_HDR_MIN_VALUE, LATENCY_HDR_MAX_VALUE, LATENCY_HDR_SIGDIGTS),
      total: 0,
    }
    times.each do |t|
      r[:total] += t
      r[:histogram].record(t)
    end
    r[:opsps] = (r[:ops].to_f / r[:total]) * 1000000
    r[:kbps] = options[:data] * r[:opsps].to_f / 1000
    r[:gbps] = (options[:data] * 8) * r[:opsps].to_f / 1000000000
    r[:p99] = r[:histogram].percentile(99)
    r[:print] =
      "ops #{r[:ops]}; total #{r[:total] / 1000}ms; ops/sec #{format(
        "%.2f",
        r[:opsps],
      )}; p99: #{r[:p99]}µs, KBps #{format("%.2f", r[:kbps])}; Gbps #{format("%.2f", r[:gbps])}"

    r
  end
end

def validate_args(options)
  options[:command] = options.fetch(:command, "set")
  valid_commands = ["set", "get"]
  raise "Invalid command. Use #{valid_commands}" unless valid_commands.find(options[:command])

  options[:runs] = options.fetch(:runs, 3)
  raise "Invalid runs. Must be a positive integer" unless options[:runs].positive?

  options[:requests] = options.fetch(:requests, 10000)
  raise "Invalid requests. Must be a positive integer" unless options[:requests].positive?

  options[:data] = options.fetch(:data, 100000)
  raise "Invalid data. Must be a positive integer" unless options[:data].positive?

  options[:ratio] = options.fetch(:ratio, 0.1)

  options[:payload] = "x" * options[:data]

  options[:server] = options.fetch(:server, "127.0.0.1")
  options[:port] = options.fetch(:port, 11211)
  options[:type] = options.fetch(:type, "dalli")
end

def run
  options = {}
  OptionParser.new do |opts|
    opts.banner = "Usage: bench [--hdr] [--stackprof] [options]"

    opts.on("-c", "--command [set/get]", String, "Stackprof against specified command. Default 'set'") do |command|
      options[:command] = command
    end

    opts.on("--stackprof", TrueClass, "Run flamegraph") do |stackprof|
      options[:stackprof] = stackprof
    end

    opts.on("-x", "--runs [NUMBER]", Integer, "Number of total bench runs to perform. Default 3") do |runs|
      options[:runs] = runs
    end

    opts.on("-n", "--requests [NUMBER]", Integer, "Number of requests to run. Default 10000") do |requests|
      options[:requests] = requests
    end

    opts.on("-d", "--data [NUMBER]", Integer, "Size of the data payload in bytes. Default 100000") do |data|
      options[:data] = data
    end

    opts.on("-r", "--ratio [NUMBER]", Float, "Ratio of ops (eg. 0.1 == 10% sets && 90% gets). Default 0.1") do |ratio|
      options[:ratio] = ratio
    end

    opts.on("-o", "--output [STRING]", String, "Output prefix for hdr files") do |out_prefix|
      options[:out_prefix] = out_prefix
    end

    opts.on("-s", "--server [STRING]", String, "Server address. Defaults to 127.0.0.1") do |server|
      options[:server] = server
    end

    opts.on("-p", "--port [NUMBER]", Integer, "Server port. Default 11211") do |port|
      options[:port] = port
    end

    opts.on("-S", "--socket [STRING]", String, "UNIX domain socket name") do |socket|
      options[:socket] = socket
    end

    opts.on("-t", "--type [STRING]", String, "Client type (dalli, lmdb), Defaults to dalli") do |type|
      options[:type] = type
    end

    opts.on("-h", "--help", "Prints this help") do
      puts(opts)
      exit
    end
  end.parse!

  validate_args(options)

  if options[:stackprof]
    puts("Running Stackprof")
    stackprof_perf(command, options[:requests])
    return
  end

  min_map = {}
  max_map = {}

  (1..options[:runs]).each do |i|
    puts "RUN: #{i}"
    bench(options).each do |op, r|
      min = min_map.fetch(op, r)
      if r[:opsps] < min[:opsps]
        min = r
      end
      min_map[op] = min

      max = max_map.fetch(op, r)
      if r[:opsps] > max[:opsps]
        max = r
      end
      max_map[op] = max
    end
    sleep(1)
  end

  puts "~~~~~~~~~~~~~~~~~~~RESULTS~~~~~~~~~~~~~~~~"
  puts "\nWORST RESULT:"
  min_map.each do |op, r|
    puts "OP: #{op} \n #{r[:print]}"
  end

  puts "\nBEST RESULT:"
  max_map.each do |op, r|
    puts "OP: #{op} \n #{r[:print]}"
    if options[:out_prefix]
      hdr_results(r[:histogram], "#{options[:out_prefix]}_rb_#{op}")
    end
  end
end

run


