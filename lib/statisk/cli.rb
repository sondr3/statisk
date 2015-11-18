require 'statisk/commands/build'
require 'statisk/commands/clean'
require 'statisk/commands/deploy'
require 'statisk/commands/init'
require 'statisk/commands/new'
require 'statisk/commands/serve'

# module Statisk
#   module CLI

#     @usage = %Q{ Usage:
#     statisk serve \tbuild and serve your site
#     statisk init \tcreate a new site
#     statisk build \tbuild your site
#     statisk clean \tcleans your built site
#     statisk deploy \tdeploy your site

#  Create content:
#     statisk new:post \tcreate a new post
#     statisk new:page \tcreate a new page
#     statisk new:draft \tcreate a new draft

#  Options:
#     -v, --version \tshow the version
#     -h, --help \t\tshow this message
#     }

#     @banner = %Q{
#  statisk -- a stupidly simple, no-nonsense static site generator
#     }

#     def self.execute
#       @options = {}

#       opts = OptionParser.new do |opts|

#         opts.banner = @banner

#         opts.on('-p', '--prod', 'Set production settings') do
#           @options[:production] = true
#         end

#         opts.on('-v', '--version', 'Show the version of Statisk') do
#           puts "Statisk #{Statisk::VERSION}"
#           exit
#         end

#         opts.on('-h', '--help', 'Help') do
#           puts @banner
#           puts @usage
#           exit
#         end

#         opts.parse!
#       end

#       if ARGV.empty?
#         puts "\n you must specify a command\n"
#         puts @usage
#         exit
#       else
#         case ARGV.first
#         when 'init'
#           Statisk::CLI::Init.new
#         when 'serve'
#           puts 'serve'
#         when 'build'
#           puts 'build'
#         when 'clean'
#           puts 'clean'
#         when 'deploy'
#           puts 'deploy'
#         when 'new:post'
#           Statisk::CLI::NewPost.new
#         when 'new:page'
#           puts 'new page'
#         when 'new:draft'
#           puts 'new draft'
#         end
#       end
#     end
#   end
# end
