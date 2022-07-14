const {REST} = require('@discordjs/rest');
const {Routes} = require('discord-api-types/v9');

const generateDaysChoices = (maxNumberOfDays) => {
  const days = [...Array(maxNumberOfDays).keys()];

  return days.map(day => (
    {
      "name": '' + (day + 1),
      "value": day + 1
    }));
}

const commands = [
  {
    name: 'played',
    description: `Get a summary of a player's recent games.`,
    type: 1,
    options: [{
        "name": "user",
        "description": "The league of legends username for the user.",
        "required": true,
        "type": 3,
        "autocomplete": true
    },{
        "name": "days",
        "description": "Over the last how many days.",
        "required": true,
        "type": 4,
        "choices": generateDaysChoices(7)
    }]
  },
  {
    name: 'ranked',
    description: `Get a summary of a player's recent ranked games.`,
    type: 1,
    options: [{
        "name": "user",
        "description": "The league of legends username for the user.",
        "required": true,
        "type": 3,
        "autocomplete": true
    },{
      "name": "days",
      "description": "Over the last how many days.",
      "required": true,
      "type": 4,
      "choices": generateDaysChoices(7)
    }]
  },
];

const rest = new REST({version: '9'}).setToken(process.env.DISCORD_TOKEN);

(async () => {
    try {
      console.log('Started refreshing application (/) commands.');
  
      await rest.put(Routes.applicationCommands(process.env.APP_ID), {
        body: commands,
      });
  
      console.log('Successfully reloaded application (/) commands.');
    } catch (error) {
      console.error(error);
    }
  })();