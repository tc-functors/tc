require 'json'
require 'thin'

require "csv"

def handler(event:, context:)
  output_string = CSV.generate('', headers: ['Name', 'Value'], write_headers: true) do |csv|
    csv << ['Foo', 0]
    csv << ['Bar', 1]
    csv << ['Baz', 2]
  end
  puts(output_string)

    { event: JSON.generate(event), context: JSON.generate(context.inspect) }
end
